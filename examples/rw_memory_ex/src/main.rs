extern crate trickster;
use trickster::Process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let ctx = Process::new("rw_memory_ex")?;

  println!();

  let kind_of_remote_var: i32 = 1337;
  println!("kind_of_remote_var before write: {}", kind_of_remote_var);
  // Read memory example:
  println!("kind_of_remote_var before write in byte buffer:");
  for byte in ctx
    .read_memory::<i32>(&kind_of_remote_var as *const i32 as usize)?
    .into_inner()
  {
    println!(" {}", byte);
  }

  // Write memory example:
  let write_buffer = vec![10u8, 0u8, 0u8, 0u8];
  ctx.write_memory::<i32>(&kind_of_remote_var as *const i32 as usize, write_buffer)?;

  println!();

  println!("kind_of_remote_var after write: {}", kind_of_remote_var);
  println!("kind_of_remote_var after write in byte buffer:");
  // Read memory example:
  for byte in ctx
    .read_memory::<i32>(&kind_of_remote_var as *const i32 as usize)?
    .into_inner()
  {
    println!(" {}", byte);
  }

  Ok(())
}
