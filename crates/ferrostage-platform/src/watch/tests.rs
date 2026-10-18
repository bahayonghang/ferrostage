use super::*;
use crate::topology::tests::monitor;

fn list(id: &str) -> Vec<MonitorInfo> {
    vec![monitor(id, true, (1920, 1080))]
}

#[test]
fn fast_changes_coalesce_but_last_committed_topology_is_preserved() {
    let now = Instant::now();
    let mut state = Observations::default();
    assert_eq!(
        state.observe(Ok(list("a")), now),
        Some(TopologyEvent::Changed(list("a")))
    );
    assert_eq!(state.observe(Ok(list("b")), now), None);
    assert_eq!(
        state.observe(Ok(list("c")), now + Duration::from_millis(100)),
        None
    );
    assert_eq!(
        state.observe(Ok(list("c")), now + Duration::from_millis(299)),
        None
    );
    assert_eq!(
        state.observe(Ok(list("c")), now + Duration::from_millis(300)),
        Some(TopologyEvent::Changed(list("c")))
    );
}

#[test]
fn error_is_immediate_diagnostic_only_and_same_topology_recovery_is_published() {
    let now = Instant::now();
    let mut state = Observations::default();
    state.observe(Ok(list("a")), now);
    let error = PlatformError::NativeEnumeration {
        operation: "fixture",
        code: 5,
    };
    assert_eq!(
        state.observe(Err(error.clone()), now),
        Some(TopologyEvent::Unavailable {
            error: error.clone(),
            last_known: list("a")
        })
    );
    assert_eq!(state.observe(Err(error), now), None);
    assert_eq!(state.observe(Ok(list("a")), now), None);
    assert_eq!(
        state.observe(Ok(list("a")), now + TOPOLOGY_POLL_INTERVAL),
        Some(TopologyEvent::Changed(list("a")))
    );
}

#[test]
fn empty_topology_is_a_change_not_an_error_and_reversal_cancels_pending() {
    let now = Instant::now();
    let mut state = Observations::default();
    state.observe(Ok(list("a")), now);
    state.observe(Ok(list("b")), now);
    assert_eq!(
        state.observe(Ok(list("a")), now + TOPOLOGY_POLL_INTERVAL),
        None
    );
    assert_eq!(
        state.observe(Ok(vec![]), now + TOPOLOGY_POLL_INTERVAL),
        None
    );
    assert_eq!(
        state.observe(Ok(vec![]), now + TOPOLOGY_POLL_INTERVAL * 2),
        Some(TopologyEvent::Changed(vec![]))
    );
}

#[test]
fn explicit_stop_and_drop_join_the_worker_without_waiting_for_another_event() {
    for explicit_stop in [true, false] {
        let (sink, received) = mpsc::channel();
        let watcher = start(sink, || Ok(vec![])).unwrap();
        assert_eq!(
            received.recv_timeout(Duration::from_secs(3)).unwrap(),
            TopologyEvent::Changed(vec![])
        );
        if explicit_stop {
            watcher.stop().unwrap();
        } else {
            drop(watcher);
        }
        assert_eq!(received.try_recv(), Err(mpsc::TryRecvError::Disconnected));
    }
}

#[test]
fn disconnected_sink_ends_worker_and_stop_remains_successful() {
    let (sink, received) = mpsc::channel();
    drop(received);
    start(sink, || Ok(vec![])).unwrap().stop().unwrap();
}
