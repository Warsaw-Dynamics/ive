/// Runs `job` on every item, spread over the available cores; results keep the items' order, and a panicking job
/// panics the caller.
pub fn in_parallel<T: Sync, R: Send>(items: &[T], job: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, |cores| cores.get());
    let per_thread = items.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = items
            .chunks(per_thread)
            .map(|batch| scope.spawn(|| batch.iter().map(&job).collect::<Vec<R>>()))
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))).collect()
    })
}
