use std::{collections::HashMap, sync::Arc, path::PathBuf, task::Poll};
use futures::{future::Shared, executor::ThreadPool, task::AtomicWaker, Future};
use futures::FutureExt;
use crossbeam::channel::{bounded, Receiver, TryRecvError};
use super::{Image, texture::Texture, image::ImageRon};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    loading: HashMap<PathBuf, Shared<TextureFuture>>,
    cache: HashMap<PathBuf, Arc<Texture>>,
}

impl TextureManager {
    pub fn new(pool: Arc<ThreadPool>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            pool,
            loading: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    #[allow(unused)]
    pub async fn load(&mut self, id: &PathBuf, img: Arc<Image>, imgron: Option<Arc<ImageRon>>) {
        if !self.cache.contains_key(id) && !self.loading.contains_key(id) {
            let mut f = TextureFuture::new(
                img,
                imgron,
                self.device.clone(),
                self.queue.clone(),
                self.pool.clone(),
                id.clone(),
            )
            .shared();
            futures::poll!(&mut f);
            self.loading.insert(id.clone(), f);
        }
    }

    pub async fn get(&mut self, id: &PathBuf) -> async_filemanager::LoadStatus<Texture, TextureFuture> {
        if let Some(f) = self.loading.get_mut(id) {
            if let Poll::Ready(result) = futures::poll!(f) {
                self.loading.remove(id);
                match result {
                    Ok(t) => {
                        self.cache.entry(id.clone()).or_insert(t.clone());
                        async_filemanager::LoadStatus::Loaded(t)
                    }
                    Err(e) => async_filemanager::LoadStatus::Error(e),
                }
            } else {
                async_filemanager::LoadStatus::Loading(self.loading.get(id).unwrap().clone())
            }
        } else if let Some(f) = self.cache.get(id) {
            async_filemanager::LoadStatus::Loaded(f.clone())
        } else {
            async_filemanager::LoadStatus::NotLoading
        }
    }
}

/// The future that resolves to a Texture
pub struct TextureFuture {
    imgdata: Arc<Image>,
    imgron: Option<Arc<ImageRon>>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    waker: Arc<AtomicWaker>,
    status: LoadStatus,
    path: PathBuf
}
#[allow(unused)]
impl TextureFuture {
    pub fn new(
        imgdata: Arc<Image>,
        imgron: Option<Arc<ImageRon>>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        pool: Arc<ThreadPool>,
        id: PathBuf,
    ) -> Self {
        Self {
            imgdata,
            imgron,
            device,
            queue,
            pool,
            waker: Arc::new(AtomicWaker::new()),
            status: LoadStatus::Image,
            path: id,
        }
    }
}

enum LoadStatus {
    Image,
    Uploading(Receiver<Arc<Texture>>),
}

impl Future for TextureFuture {
    type Output = Result<Arc<Texture>, Arc<std::io::Error>>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.status {
            LoadStatus::Image => {
                let (tx, rx) = bounded(1);
                self.waker.register(cx.waker());
                let waker = self.waker.clone();
                let imgdata = self.imgdata.clone();
                let imgron = self.imgron.clone();
                let device = self.device.clone();
                let queue = self.queue.clone();
                let path = self.path.clone();
                self.pool.spawn_ok(async move {
                    tx.send(Arc::new(Texture::new(device, queue, imgdata, imgron, path)))
                        .expect("Error forwarding loaded data!");
                    waker.wake();
                });
                self.get_mut().status = LoadStatus::Uploading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Uploading(rx) => match rx.try_recv() {
                Ok(texture) => Poll::Ready(Ok(texture)),
                Err(TryRecvError::Empty) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(Arc::new(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    e,
                )))),
            },
        }
    }
}
