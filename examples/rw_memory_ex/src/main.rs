extern crate trickster;
use trickster::external::process::Process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let ctx = Process::new("rw_memory_ex")?;

  let kind_of_remote_var: i32 = 1337;
  println!("kind_of_remote_var before write: {}", kind_of_remote_var);

  let write_buffer = vec![10u8, 0u8, 0u8, 0u8];
  ctx.write_memory::<i32>(&kind_of_remote_var as *const i32 as usize, write_buffer)?;

  println!("kind_of_remote_var after write: {}", kind_of_remote_var);

  Ok(())
}
