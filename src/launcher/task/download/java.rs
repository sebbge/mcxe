use super::DownloadTask;
use crate::Instance;

pub(crate) fn download_java(instance: Instance, download: DownloadTask, version: u16) {
  tokio::spawn(async move {
    let download_result: crate::Result<Option<()>> = async {
      //- Meta
      let package = instance.java.fetch_java_package(&version).await?;
      download.set_total(package.size).await;
      download.start().await;

      //- Download
      let jre_name = format!("jre-{}", version);
      let jre_dir = instance.java.java_dir.join(&jre_name);
      let zip_path = jre_dir.with_extension(package.ext);

      let jre_response = crate::net::get(&package.url).await?;

      crate::util::transfer::download_and_write(
        jre_response,
        &zip_path,
        || download.is_cancelled(),
        |size| download.add_bytes(size, ""),
      )
      .await?;

      //- Install
      use std::io::Read;
      #[cfg(unix)]
      use std::os::unix::fs::PermissionsExt;

      download.set_installing("Extracting Java Runtime Environment").await;
      crate::io::zip::for_each(&zip_path, |mut file| {
        let mut path_parts = file.name().splitn(2, '/');
        let _ = path_parts.next();
        let name = path_parts.next().unwrap_or("");

        if name.is_empty() {
          return Ok(());
        }

        let mut out_path = jre_dir.clone();
        out_path.push(name);

        if file.name().ends_with('/') {
          crate::io::create_dir_all_sync(&out_path)?;
        } else {
          if let Some(parent) = out_path.parent() {
            crate::io::create_dir_all_sync(parent)?;
          }

          let mut bytes = Vec::with_capacity(file.size() as usize);
          file
            .read_to_end(&mut bytes)
            .map_err(|e| crate::io::IoError::from_io(e, &zip_path))?;

          crate::io::write_sync(&out_path, &bytes)?;

          #[cfg(unix)]
          if name.contains("/bin/java") {
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&out_path, perms).map_err(|e| crate::io::IoError::from_io(e, &out_path))?;
          }
        }

        Ok(())
      })?;

      crate::io::delete_file(zip_path).await.ok();

      Ok(Some(()))
    }
    .await;

    match download_result {
      Ok(None) => download.cancel("").await,
      Ok(Some(())) => download.complete("").await,
      Err(e) => download.fail(e.to_string()).await,
    };
  });
}
