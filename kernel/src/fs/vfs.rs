use alloc::{sync::Arc, vec::Vec};
use path::Path;

const MAX_FILE_NAME: usize = 256;

struct Inode {
    filesize: u64,
}

pub struct OverlayMap {
    to: Arc<Path>,
    write_access: bool,
}

struct OverLay {
    // if no mappings are found, also check this.
    parent: Option<Arc<OverLay>>,

    // example: /home/anon
    // none: check inherits, else root
    mount_point: Option<OverlayMap>,

    // example:
    // - /exe /exe
    // Key: the path relative to the current overlay
    // Value: the absolute path
    // Its important that this vec is ordered from specific to less specific
    overlay_mappings: Vec<(Arc<Path>, OverlayMap)>,
}

impl OverLay {
    pub const ROOT: Self = OverLay {
        parent: None,
        mount_point: None,
        overlay_mappings: Vec::new(),
    };

    fn mapping_with_start_of(&self, path: &Path) -> Option<&OverlayMap> {
        self.overlay_mappings
            .iter()
            .find(|(from, _)| path.starts_with(from))
            .map(|(_, to)| to)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.mapping_with_start_of(path))
            })
    }

    fn mount_point(&self) -> Option<&OverlayMap> {
        self.mount_point
            .as_ref()
            .or_else(|| self.parent.as_ref().and_then(|p| p.mount_point()))
    }

    pub fn resolve_path<'a>(&'a self, path: &'a Path) -> (&Path, &Path) {
        assert!(path.is_absolute());

        let Some(mapping) = self
            .mapping_with_start_of(path)
            .or_else(|| self.mount_point())
        else {
            return (Path::ROOT, path);
        };

        mapping.to.combine(path)
    }
}

pub struct Vfs {}

/*

ieder proces heeft een root Overlay
en een "overlay"

ik wil dit kunnen doen:
spawn() -> zelfde root
spawn(["/home/anon", "/"])
spawn(["/home/anon/config", "/config"])


*/
