/// This describes how pages in the region can ba ccessed.  
/// There are four different permissions, lets assume that  
/// we have region with permissions == `r-xp` .  
/// Our `RegionPermissions` will have `readable` and    
/// `executable` fields set to **true**, so `writeable`  
/// and `shared` will be false, obviously.
///  
/// You can find more detailed permissions description  
/// in `MemoryRegion.permissions` field documentation.
#[derive(Eq, PartialEq, Debug)]
pub struct RegionPermissions {
  pub readable: bool,
  pub writeable: bool,
  pub executable: bool,
  pub shared: bool,
}
/// Each row in /proc/\[pid\]/maps describes a region of
/// contiguous virtual memory in a process or thread.
//  Each row has the following fields:
//  address           perms offset  dev   inode   pathname
//  08048000-08056000 r-xp 00000000 03:0c 64593   /usr/sbin/gpm
#[derive(Debug)]
pub struct MemoryRegion {
  /// This is the starting address of the region in the process's address space.
  pub start: usize,
  /// This is the ending address of the region in the process's address space.
  pub end: usize,
  /// This describes how pages in the region can be accessed.  
  /// There are four different permissions: read, write, execute, and shared.  
  /// If read/write/execute are disabled, a `-` will appear instead of the `r`/`w`/`x` .  
  /// If a region is not shared, it is private, so a `p` will appear instead of an `s` .  
  /// If the process attempts to access memory in a way that is not permitted,  
  /// a segmentation fault is generated.  
  ///  
  /// Permissions can be changed using the [**mprotect(2)**](http://man7.org/linux/man-pages/man2/mprotect.2.html) system call.
  pub permissions: RegionPermissions,
  /// If the region was mapped from a file (using mmap), this is the offset in the file  
  /// where the mapping begins. If the memory was not mapped from a file, it's just 0.
  pub offset: usize,
  /// If the region was mapped from a file, this is the  
  /// major device number (in hex) where the file lives.
  pub dev_major: u8,
  /// If the region was mapped from a file, this is the  
  /// minor device number (in hex) where the file lives.
  pub dev_minor: u8,
  /// If the region was mapped from a file, this is the file number.
  pub inode: usize,
  /// If the region was mapped from a file, this is the name of the file.  
  /// This field is [`None`] for anonymous mapped regions.  
  /// There are also special regions with names like  
  /// `[heap]`, `[stack]`, or `[vdso]` .  
  /// The last one stands for virtual dynamic shared object.  
  /// It's used by system calls to switch to kernel mode. 
  ///
  /// [`None`]: https://doc.rust-lang.org/std/option/ 
  pub path: Option<String>,
}
