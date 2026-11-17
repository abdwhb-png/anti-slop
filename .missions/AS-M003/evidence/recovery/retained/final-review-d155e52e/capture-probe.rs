use anti_slop::workspace::{capture,CaptureRequest,Selection};
use std::path::Path;
fn main(){
 let a:Vec<String>=std::env::args().collect();
 let selection=match a[2].as_str(){"staged"=>Selection::Staged,"unstaged"=>Selection::Unstaged,"local"=>Selection::Local,_=>Selection::Base("HEAD".into())};
 let r=capture(Path::new(&a[1]),&CaptureRequest::new(selection));
 match r {Ok(c)=>{println!("SUCCESS generation={} attempts={}",c.generation(),c.stats().attempts);for x in c.changes(){println!("CHANGE {:?} before={:?} after={:?}",x.path,x.before.as_ref().map(|s|(&s.path,s.role,s.git_mode,s.bytes())),x.after.as_ref().map(|s|(&s.path,s.role,s.git_mode,s.bytes())));}},Err(e)=>println!("ERROR {:?}",e)}
}
