from pathlib import Path

path = Path("phalcom-lsp/src/backend.rs")
text = path.read_text()
old = '''        if params.uri.as_str() == crate::core_documents::CORE_MODULE_URI {
            let config = self.config.read().expect("server config lock poisoned").clone();
            let roots = self
                .workspace_roots
                .read()
                .expect("workspace root lock poisoned")
                .iter()
                .filter_map(|uri| uri.to_file_path().ok())
                .collect::<Vec<_>>();
            let source = crate::core_documents::CoreSource::select(config.sysroot_path.as_deref(), &roots);
            return Ok(Some(source.text().to_string()));
        }
'''
new = '''        if params.uri.as_str() == crate::core_documents::CORE_MODULE_URI {
            if let Some(snapshot) = self.analysis.snapshot()
                && let Some(text) = snapshot.presentation_source(&phalcom_modules::ModuleId::core())
            {
                return Ok(Some(text.to_string()));
            }

            // Before the first semantic publication, retain the transport-only
            // fallback so an editor can still open the configured/bundled core
            // document. Once a snapshot exists, its presentation text is the
            // source of truth for every semantic range into `phalcom://core`.
            let config = self.config.read().expect("server config lock poisoned").clone();
            let roots = self
                .workspace_roots
                .read()
                .expect("workspace root lock poisoned")
                .iter()
                .filter_map(|uri| uri.to_file_path().ok())
                .collect::<Vec<_>>();
            let source = crate::core_documents::CoreSource::select(config.sysroot_path.as_deref(), &roots);
            return Ok(Some(source.text().to_string()));
        }
'''
if old not in text:
    raise SystemExit("missing Backend::source_text core transport anchor")
path.write_text(text.replace(old, new, 1))
