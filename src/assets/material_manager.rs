use std::{collections::HashMap, sync::Arc, path::PathBuf, task::Poll};
use futures::{future::Shared, executor::ThreadPool, task::AtomicWaker, Future};
use futures::FutureExt;
use crossbeam::channel::{bounded, Receiver, TryRecvError};
use super::{texture::Texture, material::{BindMaterial, PBRMaterial, PBRMaterialRon, Material}};

pub struct MaterialManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pool: Arc<ThreadPool>,
    loading: HashMap<PathBuf, Shared<MaterialFuture<PBRMaterialRon, PBRMaterial>>>,
    cache: HashMap<PathBuf, Arc<dyn BindMaterial>>,
}

impl MaterialManager {
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
    pub async fn load(&mut self, id: &PathBuf, material: Arc<PBRMaterialRon>, textures: Vec<Arc<Texture>>) {
        if !self.cache.contains_key(id) && !self.loading.contains_key(id) {
            let mut f = MaterialFuture::<PBRMaterialRon, PBRMaterial>::new(
                textures,
                material,
                self.device.clone(),
                self.pool.clone(),
                id.clone(),
            )
            .shared();
            futures::poll!(&mut f);
            self.loading.insert(id.clone(), f);
        }
    }

    // pub async fn get(&mut self, id: &PathBuf) -> async_filemanager::LoadStatus<Texture, TextureFuture> {
    //     if let Some(f) = self.loading.get_mut(id) {
    //         if let Poll::Ready(result) = futures::poll!(f) {
    //             self.loading.remove(id);
    //             match result {
    //                 Ok(t) => {
    //                     self.cache.entry(id.clone()).or_insert(t.clone());
    //                     async_filemanager::LoadStatus::Loaded(t)
    //                 }
    //                 Err(e) => async_filemanager::LoadStatus::Error(e),
    //             }
    //         } else {
    //             async_filemanager::LoadStatus::Loading(self.loading.get(id).unwrap().clone())
    //         }
    //     } else if let Some(f) = self.cache.get(id) {
    //         async_filemanager::LoadStatus::Loaded(f.clone())
    //     } else {
    //         async_filemanager::LoadStatus::NotLoading
    //     }
    // }
}

/// The future that resolves to a Texture
pub struct MaterialFuture<T, T2> {
    textures: Vec<Arc<Texture>>,
    material: Arc<T>,
    device: Arc<wgpu::Device>,
    pool: Arc<ThreadPool>,
    waker: Arc<AtomicWaker>,
    status: LoadStatus<T2>,
    path: PathBuf
}
#[allow(unused)]
impl<T, T2> MaterialFuture<T, T2> 
where T: Material<T2>, T2: BindMaterial {
    pub fn new(
        textures: Vec<Arc<Texture>>,
        material: Arc<T>,
        device: Arc<wgpu::Device>,
        pool: Arc<ThreadPool>,
        id: PathBuf,
    ) -> Self {
        Self {
            textures,
            material,
            device,
            pool,
            waker: Arc::new(AtomicWaker::new()),
            status: LoadStatus::Image,
            path: id,
        }
    }
}

enum LoadStatus<T> {
    Image,
    Uploading(Receiver<Arc<T>>),
}

impl<T, T2> Future for MaterialFuture<T, T2>
where T: Material<T2> + Send + Sync + 'static, T2: BindMaterial + Send + Sync + 'static {
    type Output = Result<Arc<T2>, Arc<std::io::Error>>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &self.status {
            LoadStatus::Image => {
                let (tx, rx) = bounded(1);
                self.waker.register(cx.waker());
                let waker = self.waker.clone();
                let device = self.device.clone();
                let path = self.path.clone();
                let material = self.material.clone();
                let textures = self.textures.clone();
                self.pool.spawn_ok(async move {
                    let bind_material = material.create_material(textures);
                    tx.send(Arc::new(bind_material))
                        .expect("Error forwarding loaded data!");
                    waker.wake();
                });
                self.get_mut().status = LoadStatus::Uploading(rx);
                std::task::Poll::Pending
            }
            LoadStatus::Uploading(rx) => match rx.try_recv() {
                Ok(bind_material) => Poll::Ready(Ok(bind_material)),
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
