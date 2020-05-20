use anyhow::{anyhow, Result};
use nix::sys::uio::{process_vm_readv, process_vm_writev, IoVec, RemoteIoVec};
use nix::unistd::Pid;
use std::fs;
use std::io::Cursor;
use std::mem;

/// # Process
/// Process is object implementation of existing numeric entry  
/// in `/proc/` directory.
pub struct Process {
  pid: Pid,
  name: String,
}

impl Process {
  /// Process object constructor. Finds process id by name by iterating  
  /// over numeric directories in `/proc/` and comparing name  
  /// provided in method parameter with one located in `/proc/\[pid\]/comm` file.
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
  /// ...
  /// let mut read_byte_buffer = ctx.read_memory::<i32>(&kind_of_remote_var as *const i32 as usize)?;
  /// println!("kind_of_remote_var from byte buffer: {}", read_byte_buffer.read_i32::<LittleEndian>()?);
  /// ...
  /// ```
  /// ...and this prints output like:  
  /// `example process id: 26444`  
  /// `kind_of_remote_var from byte buffer: 1337`
  pub fn read_memory<T>(&self, address: usize) -> Result<Cursor<Vec<u8>>> {
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
          ))
        }
      };

    if bytes_read != bytes_requested {
      return Err(anyhow!("Could not read memory. Partial read occurred."));
    }

    Ok(Cursor::new(buffer))
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
  /// ...
  /// let mut write_buffer = Vec::new();
  /// write_buffer.write_i32::<LittleEndian>(10)?;
  /// ctx.write_memory::<i32>(&kind_of_remote_var as *const i32 as usize, write_buffer)?;
  /// ...
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
        ))
      }
    };

    if bytes_written != bytes_requested {
      return Err(anyhow!("Could not write memory. Partial write occurred."));
    }

    Ok(())
  }

  /// Returns copy of process id.
  pub fn get_pid(&self) -> Pid {
    self.pid
  }

  /// Returns immutable reference to the process name.
  pub fn get_name(&self) -> &String {
    &self.name
  }
}
