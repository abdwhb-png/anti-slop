use anti_slop::workspace::{capture, CaptureRequest, GitMode, Role, Selection, SourceRange};
use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let paired = args[2] == "paired";
    for selection in [Selection::Unstaged, Selection::Local, Selection::Base("HEAD".into())] {
        let request = CaptureRequest {
            context: vec!["moved.ts".into()],
            ..CaptureRequest::new(selection)
        };
        let result = capture(Path::new(&args[1]), &request).unwrap();
        assert_eq!(result.stats().attempts, 1);
        assert_eq!(result.changes().len(), if paired { 1 } else { 2 });
        let added = result.changes().iter().find(|c| c.path == "moved.ts").unwrap();
        let after = added.after.as_ref().unwrap();
        assert_eq!(after.role, Role::Working);
        assert_eq!(after.bytes(), b"same bytes\n");
        assert_eq!(after.git_mode, Some(GitMode::Regular));
        if paired {
            let before = added.before.as_ref().unwrap();
            assert_eq!(before.path, "a.ts");
            assert_eq!(before.bytes(), after.bytes());
            assert_eq!(before.git_mode, after.git_mode);
        } else {
            assert!(added.before.is_none());
            let deleted = result.changes().iter().find(|c| c.path == "a.ts").unwrap();
            assert!(deleted.after.is_none());
            assert_eq!(deleted.before.as_ref().unwrap().git_mode, Some(GitMode::Executable));
        }
        let context = &result.context()[0];
        assert_eq!(context.bytes(), after.bytes());
        assert_eq!(context.git_mode, after.git_mode);
        assert_ne!(context.generation, after.generation);
        assert_eq!(result.slice(&SourceRange {
            path: "moved.ts".into(), role: Role::Working,
            generation: after.generation.clone(), range: 0..after.bytes().len()
        }).unwrap(), after.bytes());
        assert_eq!(result.generation(), capture(Path::new(&args[1]), &request).unwrap().generation());
        println!("PASS {:?} paired={paired} changes={}", request.selection, result.changes().len());
    }
}
