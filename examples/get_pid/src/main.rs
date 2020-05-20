extern crate trickster;
use trickster::external::process::Process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let ctx = Process::new("get_pid")?;
  println!("example process id: {}", ctx.get_pid());
  Ok(())
}
