use anyhow::Result;
use nix::sys::uio::{process_vm_readv, process_vm_writev, IoVec, RemoteIoVec};
use nix::unistd::Pid;
use std::fs;
use std::io::{self, BufRead};
use std::mem;
use std::path;

use super::{MemoryRegion, RegionPermissions};

// TODO: Document rest of fields
/// Process is an object implementation of existing   
/// numeric entry in `/proc/` directory.
///   
/// **NOTE**: `memory_regions` field can be [`None`] .     
/// if memory regions were not mapped. (`parse_maps()` was not called).
///   
/// [`None`]: https://doc.rust-lang.org/std/option/
pub struct Process {
  /// A Pid (i.e., process identification number) is an auto   
  /// generated identification number for each process.
  pid: Pid,
  name: String,
  memory_regions: Option<Vec<MemoryRegion>>,
}

impl Process {
  /// Process object constructor. Finds process id by name by iterating  
  /// over numeric directories in `/proc/` and comparing name  
  /// provided in method parameter with one located in `/proc/\[pid\]/comm` file.
  ///
  /// **WARNING**: This method __does not__ initialize `memory_regions` field.  
  /// If you want to do so, use `parse_maps()`.
  ///
  /// # Examples
  /// ```
  /// extern crate trickster;
  /// use trickster::external::Process;
  ///
  /// fn main() -> Result<(), Box<dyn std::error::Error>> {
  ///   let ctx = Process::new("current_process_name")?;
  ///   println!("example process id: {}", ctx.get_pid());
  ///   Ok(())
  /// }
  /// ```
  /// This prints output like:  
  /// `example process id: 2686`
  ///
  /// # Note about /proc/\[pid\]/comm
  /// This file exposes the process's comm value — that is, the com‐  
  /// mand name associated with the process.  Different threads in  
  /// the same process may have different comm values, accessible  
  /// via `/proc/[pid]/task/\[tid\]/comm`.  A thread may modify its comm  
  /// value, or that of any of other thread in the same thread group  
  /// (see the discussion of **CLONE_THREAD** in
  /// [**clone(2)**](http://man7.org/linux/man-pages/man2/clone.2.html)), by writing  
  /// to the file `/proc/self/task/\[tid\]/comm`. Strings longer than  
  /// **TASK_COMM_LEN** (16) characters are silently truncated.  
  ///
  /// This file provides a superset of the
  /// [**prctl(2)**](http://man7.org/linux/man-pages/man2/prctl.2.html) **PR_SET_NAME** and  
  /// **PR_GET_NAME** operations, and is employed by  
  /// [**pthread_setname_np(3)**](http://man7.org/linux/man-pages/man3/pthread_setname_np.3.html) when used to rename  
  /// threads other than the caller.  
  pub fn new(process_name: &str) -> Result<Process> {
    let process_list = fs::read_dir("/proc/")?;

    for process in process_list.filter_map(|process| process.ok()) {
      let filename_string = process
        .file_name()
        .into_string()
        .map_err(|_| anyhow!("Could not convert file_name() of process directory to String."))?;

      if !filename_string.chars().all(char::is_numeric) {
        continue;
      }

      let comm_path = process.path().join("comm");
      let true_name = fs::read_to_string(comm_path)?;

      if true_name.trim_end() == process_name.to_string() {
        return Ok(Process {
          pid: Pid::from_raw(
            filename_string
              .parse::<i32>()
              .expect("Could not parse i32 value from filename_string."),
          ),
          name: true_name,
          memory_regions: None,
        });
      }
    }

    Err(anyhow!("Could not get process id of {}.", process_name))
  }
  /// Returns [`Cursor`] wrapping around byte buffer containing memory read at `address`  
  /// in remote process. Size of returned byte buffer is equivalent to size of generic type (`T`).  
  /// Reading is done using [**process_vm_readv(2)**](http://man7.org/linux/man-pages/man2/process_vm_readv.2.html)
  /// system call.
  ///
  /// This requires the same permissions as debugging the process using
  /// [**ptrace(2)**](http://man7.org/linux/man-pages/man2/ptrace.2.html): you must either be  
  /// a privileged process (with **CAP_SYS_PTRACE**), or you must be running as the same user as the target  
  /// process and the OS must have unprivileged debugging enabled.
  ///
  /// [`Cursor`]: https://doc.rust-lang.org/std/io/struct.Cursor.html
  ///
  /// # Examples
  /// NOTE: In this scenario, process running this code is treated as remote process.  
  /// Just for example purposes.
  /// ```
  /// extern crate trickster;
  /// use trickster::external::Process;
  ///
  /// fn main() -> Result<(), Box<dyn std::error::Error>> {
  ///   let ctx = Process::new("current_process_name")?;
  ///   println!("example process id: {}", ctx.get_pid());
  ///
  ///   let kind_of_remote_var: i32 = 1337;
  ///   let read_byte_buffer = ctx.read_memory::<i32>(&kind_of_remote_var as *const i32 as usize)?;
  ///
  ///   for (index, byte) in read_byte_buffer.into_inner().iter().enumerate() {
  ///     println!("read_byte_buffer[{}]: {}", index, byte);
  ///   }
  ///
  ///   Ok(())
  /// }
  /// ```
  /// This prints output like:  
  /// `example process id: 25805`  
  /// `read_byte_buffer[0]: 57`  
  /// `read_byte_buffer[1]: 5`  
  /// `read_byte_buffer[2]: 0`  
  /// `read_byte_buffer[3]: 0`  
  ///
  ///
  /// If you want to construct 'real' value from byte buffer, using [`byteorder`] crate  
  /// would be the easiest way because this action is kinda difficult to standardize  
  /// because of [endianness](https://en.wikipedia.org/wiki/Endianness).
  ///
  /// [`byteorder`]: https://crates.io/crates/byteorder
  ///
  /// Changing (a little) code above would look like:
  /// ```
  /// extern crate byteorder;
  /// use byteorder::{LittleEndian, ReadBytesExt};
  /// // ...
  /// let mut read_byte_buffer = ctx.read_memory::<i32>(&kind_of_remote_var as *const i32 as usize)?;
  /// println!("kind_of_remote_var from byte buffer: {}", read_byte_buffer.read_i32::<LittleEndian>()?);
  /// // ...
  /// ```
  /// ...and this prints output like:  
  /// `example process id: 26444`  
  /// `kind_of_remote_var from byte buffer: 1337`
  pub fn read_memory<T>(&self, address: usize) -> Result<io::Cursor<Vec<u8>>> {
    let bytes_requested = mem::size_of::<T>();
    let mut buffer = vec![0u8; bytes_requested];

    let remote = RemoteIoVec {
      base: address,
      len: bytes_requested,
    };

    let bytes_read =
      match process_vm_readv(self.pid, &[IoVec::from_mut_slice(&mut buffer)], &[remote]) {
        Ok(bytes_read) => bytes_read,
        Err(error) => {
          return Err(anyhow!(
            "Could not read memory at {:#x} ({}).",
            address,
            error
          ));
        }
      };

    if bytes_read != bytes_requested {
      return Err(anyhow!("Could not read memory. Partial read occurred."));
    }

    Ok(io::Cursor::new(buffer))
  }

  /// Writes `buffer` at `address` in remote process. Size of `buffer`  
  /// is (or should be, if specified) equivalent to size of generic type (`T`).  
  /// Writing is done using [**process_vm_writev(2)**](http://man7.org/linux/man-pages/man2/process_vm_writev.2.html)
  /// system call.
  ///
  /// This requires the same permissions as debugging the process using
  /// [**ptrace(2)**](http://man7.org/linux/man-pages/man2/ptrace.2.html): you must either be  
  /// a privileged process (with **CAP_SYS_PTRACE**), or you must be running as the same user as the target  
  /// process and the OS must have unprivileged debugging enabled.
  ///
  /// # Examples
  /// NOTE: In this scenario, process running this code is treated as remote process.  
  /// Just for example purposes.
  /// ```
  /// extern crate trickster;
  /// use trickster::external::Process;
  ///   
  /// fn main() -> Result<(), Box<dyn std::error::Error>> {
  ///   let ctx = Process::new("current_process_name")?;
  ///   println!("example process id: {}", ctx.get_pid());
  ///
  ///   let kind_of_remote_var: i32 = 1337;
  ///   println!("kind_of_remote_var before write: {}", kind_of_remote_var);
  ///
  ///   let write_buffer = vec![10u8, 0u8, 0u8, 0u8];
  ///   ctx.write_memory::<i32>(&kind_of_remote_var as *const i32 as usize, write_buffer)?;
  ///
  ///   println!("kind_of_remote_var after write: {}", kind_of_remote_var);
  ///
  ///   Ok(())
  /// }
  /// ```
  /// This prints output like:  
  /// `example process id: 25805`  
  /// `kind_of_remote_var before write: 1337`  
  /// `kind_of_remote_var after write: 10`  
  ///
  /// If you want to construct 'real' value from byte buffer, using [`byteorder`] crate  
  /// would be the easiest way because this action is kinda difficult to standardize  
  /// because of [endianness](https://en.wikipedia.org/wiki/Endianness).
  ///
  /// [`byteorder`]: https://crates.io/crates/byteorder
  ///
  /// Changing (a little) code above to would look like:
  /// ```
  /// extern crate byteorder;
  /// use byteorder::{LittleEndian, WriteBytesExt};
  /// // ...
  /// let mut write_buffer = Vec::new();
  /// write_buffer.write_i32::<LittleEndian>(10)?;
  /// ctx.write_memory::<i32>(&kind_of_remote_var as *const i32 as usize, write_buffer)?;
  /// // ...
  /// ```
  pub fn write_memory<T>(&self, address: usize, buffer: Vec<u8>) -> Result<()> {
    let bytes_requested = mem::size_of::<T>();

    let remote = RemoteIoVec {
      base: address,
      len: bytes_requested,
    };

    let bytes_written = match process_vm_writev(self.pid, &[IoVec::from_slice(&buffer)], &[remote])
    {
      Ok(bytes_written) => bytes_written,
      Err(error) => {
        return Err(anyhow!(
          "Could not write memory at {:#x} ({}).",
          address,
          error
        ));
      }
    };

    if bytes_written != bytes_requested {
      return Err(anyhow!("Could not write memory. Partial write occurred."));
    }

    Ok(())
  }

  /// Reads `/proc/\[pid\]/maps` file line by line and parses  
  /// every value to the corresponding value in `MemoryRegion` struct  
  /// in `self.memory_regions`.
  pub fn parse_maps(&mut self) -> Result<()> {
    let maps_path = path::Path::new("/proc/")
      .join(self.pid.to_string())
      .join("maps");

    let mut reader = io::BufReader::new(fs::File::open(maps_path)?);
    let mut buffer = Vec::<u8>::new();
    let mut memory_regions: Vec<MemoryRegion> = Vec::new();

    while reader.read_until(b'\n', &mut buffer)? != 0 {
      let line = String::from_utf8(buffer).unwrap();
      let mut permissions: RegionPermissions = RegionPermissions {
        readable: false,
        writeable: false,
        executable: false,
        shared: false,
      };

      let (start, end, permissions_string, offset, dev_major, dev_minor, inode, path) = scan_fmt_some!(
        line.as_str(),
        "{x}-{x} {} {x} {}:{} {} {}",
        [hex usize], [hex usize], String, [hex usize], u8, u8, usize, String
      );

      for character in permissions_string.unwrap().chars() {
        match character {
          'r' => permissions.readable = true,
          'w' => permissions.writeable = true,
          'x' => permissions.executable = true,
          's' => permissions.shared = true,
          _ => continue,
        }
      }

      memory_regions.push(MemoryRegion {
        start: start.unwrap(),
        end: end.unwrap(),
        permissions,
        offset: offset.unwrap(),
        dev_major: dev_major.unwrap(),
        dev_minor: dev_minor.unwrap(),
        inode: inode.unwrap(),
        path,
      });

      buffer = line.into_bytes();
      buffer.clear();
    }

    self.memory_regions = Some(memory_regions);

    Ok(())
  }

  /// Returns process id.
  pub fn get_pid(&self) -> Pid {
    self.pid
  }

  /// Returns process name.
  pub fn get_name(&self) -> &String {
    &self.name
  }

  /// Returns immutable reference to the memory regions.  
  /// If `self.memory_regions` is [`None`], [`Err`] is returned.  
  ///
  /// [`None`]: https://doc.rust-lang.org/std/option/
  /// [`Err`]: https://doc.rust-lang.org/std/result/
  ///  
  /// **NOTE**: `parse_maps();` should be called minimum once  
  /// before calling `get_memory_regions();`.
  pub fn get_memory_regions(&self) -> Result<&Vec<MemoryRegion>> {
    return match &self.memory_regions {
      Some(memory_regions) => Ok(memory_regions),
      None => Err(anyhow!("Memory regions not mapped.")),
    };
  }

  /// Returns immutable reference to memory region with  
  /// `path` field in `MemoryRegion` struct trimmed to  
  /// contain only file name equals `region_name` and  
  /// region permissions equals `permissions_eq` if not [`None`].  
  ///   
  /// [`None`]: https://doc.rust-lang.org/std/option/
  ///  
  /// **NOTES**:
  /// - `parse_maps();` should be called minimum once  
  /// before calling `region_find_first_by_name();`.
  /// - `region_name` can be equal to `[anonymous_region]` if  
  /// region was not mapped from a file or its not special.
  pub fn region_find_first_by_name(
    &self,
    region_name: &str,
    permissions_eq: Option<RegionPermissions>,
  ) -> Result<&MemoryRegion> {
    let regions = self.get_memory_regions()?;
    for region in regions {
      let index_to_split = region
        .path
        .clone()
        .unwrap_or("[anonymous_region]".to_string())
        .rfind('/')
        .unwrap_or(0 as usize);

      let split_file_name = region
        .path
        .clone()
        .unwrap_or("[anonymous_region]".to_string())
        .split_off(index_to_split + if index_to_split > 0 { 1 } else { 0 });

      if split_file_name == region_name {
        return match permissions_eq {
          Some(permissions) => {
            if permissions == region.permissions {
              Ok(region)
            } else {
              Err(anyhow!("Could not get region with specific permissions."))
            }
          }
          None => Ok(region),
        };
      }
    }
    Err(anyhow!("Could not find {}.", region_name))
  }

  /// Returns the region in which's range `address` is located.  
  /// If `self.memory_regions` is [`None`], [`Err`] is returned.  
  ///
  /// [`None`]: https://doc.rust-lang.org/std/option/
  /// [`Err`]: https://doc.rust-lang.org/std/result/
  ///  
  /// **NOTE**: `parse_maps();` should be called minimum once  
  /// before calling `get_memory_regions();`.
  pub fn get_address_region(&self, address: usize) -> Result<&MemoryRegion> {
    match &self.memory_regions {
      Some(regions) => {
        for region in regions {
          if address >= region.start && address <= region.end {
            return Ok(region);
          }
        }
      }
      None => return Err(anyhow!("Memory regions not mapped.")),
    }
    Err(anyhow!("Could not get {:x}'s region.", address))
  }

  // TODO: document this
  #[cfg(feature = "byteorder-utils")]
  #[cfg(target_endian = "little")]
  /// Returns the absolute address calculated from function parameters.
  pub fn abs_addr(&self, address: usize, offset: usize, size: usize) -> Result<usize> {
    use byteorder::{NativeEndian, ReadBytesExt};
    if let Ok(mut buffer) = self.read_memory::<u32>(address + offset) {
      let value = buffer.read_u32::<NativeEndian>()?;
      return Ok(value as usize + address + size);
    }
    Err(anyhow!("Could not get absolute address."))
  }

  // TODO: document this
  #[cfg(feature = "byteorder-utils")]
  #[cfg(target_endian = "little")]
  /// Returns the call address.
  pub fn call_addr(&self, address: usize) -> Result<usize> {
    use byteorder::{NativeEndian, ReadBytesExt};
    if let Ok(mut buffer) = self.read_memory::<u32>(address + 0x1) {
      let value = buffer.read_u32::<NativeEndian>()?;
      return Ok(value as usize + address + 0x5);
    }
    Err(anyhow!("Could not get call address."))
  }
}
