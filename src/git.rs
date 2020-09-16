pub(crate) struct Repository(pub(crate) Git2Repository);
use anyhow::Result;
use git2::{
    Diff, DiffOptions, IndexAddOption, Object, ObjectType, Oid,
    Repository as Git2Repository, StatusOptions, Statuses,
};

impl Repository {
    pub(crate) fn open() -> Result<Repository> {
        let repo = Git2Repository::discover(".")?;
        Ok(Repository(repo))
    }

    pub(crate) fn add_all(&self) -> Result<()> {
        let mut index = self.0.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write().map_err(|err| anyhow!(err))
    }


    pub(crate) fn get_diff(&self) -> Option<Diff> {
        let mut options = DiffOptions::new();

        let diff = if let Some(head) = &self.get_head() {
            self.0
                .diff_tree_to_index(head.as_tree(), None, Some(&mut options))
        } else {
            self.0
                .diff_tree_to_workdir_with_index(None, Some(&mut options))
        };

        if let Ok(diff) = diff {
            if diff.deltas().len() > 0 {
                Some(diff)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn get_statuses(&self) -> Result<Statuses> {
        let mut options = StatusOptions::new();
        options.include_untracked(true);
        options.exclude_submodules(true);
        options.include_unmodified(false);

        self.0
            .statuses(Some(&mut options))
            .map_err(|err| anyhow!(err))
    }

    pub(crate) fn commit(&self, message: String) -> Result<Oid> {
        let sig = self.0.signature()?;
        let tree_id = self.0.index()?.write_tree()?;
        let tree = self.0.find_tree(tree_id)?;
        let is_empty = self.0.is_empty()?;
        let has_delta = self.get_diff().is_some();

        if !is_empty && has_delta {
            let head = &self.0.head()?;
            let head_target = head.target().expect("Cannot get HEAD target");
            let tip = &self.0.find_commit(head_target)?;

            self.0
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&tip])
                .map_err(|err| anyhow!(err))
        } else if is_empty && has_delta {
            // First repo commit
            self.0
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])
                .map_err(|err| anyhow!(err))
        } else {
            let statuses = self.get_statuses()?;
            statuses.iter().for_each(|status| {
                eprintln!("{} : {:?}", status.path().unwrap(), status.status());
            });

            Err(anyhow!("{} : {} ", is_empty, has_delta))
        }
    }

    pub(crate) fn get_head(&self) -> Option<Object> {
        if let Ok(head) = Repository::tree_to_treeish(&self.0, Some(&"HEAD".to_string())) {
            head
        } else {
            None
        }
    }

    fn tree_to_treeish<'a>(
        repo: &'a Git2Repository,
        arg: Option<&String>,
    ) -> Result<Option<Object<'a>>, git2::Error> {
        let arg = match arg {
            Some(s) => s,
            None => return Ok(None),
        };
        let obj = repo.revparse_single(arg)?;
        let tree = obj.peel(ObjectType::Tree)?;
        Ok(Some(tree))
    }
}