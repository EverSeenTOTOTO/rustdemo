use argh::FromArgs;
use async_std::{fs::File, io::ReadExt};

use color_eyre::eyre;
use futures::io::AsyncRead;
use sha3::Digest;
use std::path::{Path, PathBuf};

/// Prints the SHA3-256 hash of some files
#[derive(FromArgs)]
struct Args {
    /// the files whose contents to hash and print
    #[argh(positional)]
    files: Vec<PathBuf>,
}

pub async fn test_hash() -> Result<(), eyre::Error> {
    let args: Args = argh::from_env();

    let mut handles = Vec::new();

    for file in &args.files {
        let file = file.clone();
        let handle = async_std::task::spawn(async move {
            let res = hash_file(&file).await;
            if let Err(e) = res {
                println!("While hashing {}: {}", file.display(), e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await;
    }

    Ok(())
}

async fn hash_file(path: &Path) -> Result<(), eyre::Error> {
    let file = File::open(path).await?;
    let mut file = TracingReader { inner: file };
    let mut hasher = sha3::Sha3_256::new();

    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf[..]).await?;
        match n {
            0 => break,
            n => hasher.update(&buf[..n]),
        }
    }

    let hash = hasher.finalize();
    print!("{} ", path.display());
    for x in hash {
        print!("{:02x}", x);
    }
    println!();

    Ok(())
}

struct TracingReader<R>
where
    R: AsyncRead,
{
    inner: R,
}

impl<R> AsyncRead for TracingReader<R>
where
    R: AsyncRead,
{
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // tracing
        let address = &self as *const _;
        println!("{:?} => {:?}", address, std::thread::current().id());

        // reading - pinning is structural for `self.inner`
        let inner: std::pin::Pin<&mut R> = unsafe { self.map_unchecked_mut(|x| &mut x.inner) };
        inner.poll_read(ctx, buf)
    }
}
