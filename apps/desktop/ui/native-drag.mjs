// Same serialized start/end protocol as RepoTicker: physical Wayland
// pointer deltas belong to the compositor, never JavaScript coordinate math.
export function createNativeDragGesture({start, stop, onError}) {
  let active = null;
  let queue = Promise.resolve();
  function enqueue(operation) {
    const job = queue.then(operation);
    queue = job.catch(() => {});
    return job;
  }
  return {
    begin(id) {
      if (active !== null) return false;
      const session = {id};
      active = session;
      void enqueue(start).catch(error => {
        if (active === session) active = null;
        onError(error);
      });
      return true;
    },
    end(id = null) {
      if (active === null || (id !== null && active.id !== id)) return;
      active = null;
      void enqueue(stop).catch(onError);
    }
  };
}
