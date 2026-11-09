//! Make way's "near" zone: is the pointer close to the island (on its way to a browser's tabs,
//! say)? The island drops to a line while it is.
//!
//! Two things keep it from flickering:
//! - The zone is measured from the full island, never from the line it turned into. Measured
//!   from the line, the zone shrank as soon as the island made way, the pointer was suddenly
//!   outside it, the island came back, the zone grew, and round it went.
//! - Leaving takes more than arriving: the zone is 48 px wider once the pointer is near, and the
//!   pointer has to stay out of it for 250 ms before the island comes back.

/// How far the zone reaches past the island, in CSS px: either side, and below.
const SIDE: f64 = 160.0;
const BELOW: f64 = 120.0;
/// Extra reach once near, so a pointer resting on the edge doesn't toggle it.
const SLACK: f64 = 48.0;
/// How long the pointer must stay out before the island comes back.
const LEAVE_MS: u64 = 250;

/// The full island's edges on screen, physical px.
#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
    /// Top of the monitor: the zone runs from here down.
    pub top: f64,
    /// Monitor scale (CSS px to physical px).
    pub scale: f64,
}

#[derive(Default)]
pub struct Near {
    on: bool,
    out_since: Option<u64>,
}

impl Near {
    pub fn on(&self) -> bool {
        self.on
    }

    /// Feeds one pointer reading (`now` in ms, any steady clock). Returns the new state when it
    /// changes, for the caller to send to the page.
    pub fn update(&mut self, x: f64, y: f64, zone: Zone, now: u64) -> Option<bool> {
        let grow = if self.on { SLACK } else { 0.0 };
        let s = zone.scale;
        let inside = x >= zone.left - (SIDE + grow) * s
            && x <= zone.right + (SIDE + grow) * s
            && y <= zone.bottom + (BELOW + grow) * s
            && y >= zone.top - 4.0;
        if inside {
            self.out_since = None;
            if !self.on {
                self.on = true;
                return Some(true);
            }
            return None;
        }
        if !self.on {
            return None;
        }
        let since = *self.out_since.get_or_insert(now);
        if now.saturating_sub(since) >= LEAVE_MS {
            self.on = false;
            self.out_since = None;
            return Some(false);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A full island 300 px wide and 34 px tall, centred on a 1920 px screen at 100%.
    const ZONE: Zone = Zone {
        left: 810.0,
        right: 1110.0,
        bottom: 34.0,
        top: 0.0,
        scale: 1.0,
    };

    #[test]
    fn arriving_turns_it_on_at_once() {
        let mut n = Near::default();
        assert_eq!(n.update(960.0, 300.0, ZONE, 0), None);
        assert_eq!(n.update(960.0, 150.0, ZONE, 16), Some(true));
    }

    #[test]
    fn a_pointer_resting_on_the_edge_does_not_flicker() {
        // The reported bug: resting about 125 px down. Measured from the full island (the zone
        // no longer shrinks with the line), it stays near, with no toggling.
        let mut n = Near::default();
        let mut changes = 0;
        for t in 0..200 {
            if n.update(1010.0, 150.0, ZONE, t * 16).is_some() {
                changes += 1;
            }
        }
        assert_eq!(changes, 1);
        assert!(n.on());
    }

    #[test]
    fn leaving_needs_the_slack_and_a_pause() {
        let mut n = Near::default();
        n.update(960.0, 150.0, ZONE, 0);
        // Just past the arriving edge (154) but inside the slack (202): still near.
        assert_eq!(n.update(960.0, 190.0, ZONE, 16), None);
        assert!(n.on());
        // Out past the slack: not yet, only after 250 ms out.
        assert_eq!(n.update(960.0, 260.0, ZONE, 32), None);
        assert_eq!(n.update(960.0, 260.0, ZONE, 200), None);
        assert_eq!(n.update(960.0, 260.0, ZONE, 290), Some(false));
    }

    #[test]
    fn coming_back_during_the_pause_keeps_it_on() {
        let mut n = Near::default();
        n.update(960.0, 150.0, ZONE, 0);
        n.update(960.0, 260.0, ZONE, 16);
        assert_eq!(n.update(960.0, 150.0, ZONE, 150), None);
        assert_eq!(n.update(960.0, 260.0, ZONE, 300), None);
        assert!(n.on());
    }
}
