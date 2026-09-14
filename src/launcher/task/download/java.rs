use super::{DownloadStatusExt, DownloadTask};
use crate::Instance;

pub(crate) fn download_java(instance: Instance, download: DownloadTask, version: u16) {
  tokio::spawn(async move {
    let download_result: crate::Result<Option<()>> = async {
      let package = instance.java.fetch_java_package(&version).await?;

      download.set_total(package.size);
      download.set_downloading();
      download.set_message(format!("Java {version}"));
      download.start();

      let jre_name = format!("jre-{}", version);
      let jre_dir = instance.java.java_dir.join(&jre_name);
      let zip_path = jre_dir.with_extension(package.ext);

      let jre_res = crate::net::get(&package.url).await?;
      crate::util::stream_to_file::<crate::Error>(
        jre_res,
        &zip_path,
        || download.is_aborted(),
        |size| download.advance(size as u64),
      )
      .await?;

      use std::io::Read;
      #[cfg(unix)]
      use std::os::unix::fs::PermissionsExt;

      download.set_installing();
      crate::fs::zip::for_each(&zip_path, |mut file| {
        let mut path_parts = file.name().splitn(2, '/');
        let _ = path_parts.next();
        let name = path_parts.next().unwrap_or("");

        if name.is_empty() {
          return Ok(());
        }

        let mut out_path = jre_dir.clone();
        out_path.push(name);

        if file.name().ends_with('/') {
          crate::fs::create_dir_all_sync(&out_path)?;
        } else {
          if let Some(parent) = out_path.parent() {
            crate::fs::create_dir_all_sync(parent)?;
          }

          let mut bytes = Vec::with_capacity(file.size() as usize);
          file.read_to_end(&mut bytes).map_err(|e| crate::fs::IoError::from_io(e, &zip_path))?;

          crate::fs::write_sync(&out_path, &bytes)?;

          #[cfg(unix)]
          if name.contains("/bin/java") {
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&out_path, perms).map_err(|e| crate::fs::IoError::from_io(e, &out_path))?;
          }
        }

        Ok(())
      })?;

      crate::fs::delete_file(zip_path).await.ok();

      Ok(Some(()))
    }
    .await;

    match download_result {
      Ok(None) => download.aborted(),
      Ok(Some(())) => download.success(),
      Err(e) => {
        download.set_error_message(e.to_string());
        download.failed()
      }
    };
  });
}
