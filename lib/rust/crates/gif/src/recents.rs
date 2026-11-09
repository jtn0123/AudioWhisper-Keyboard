//! The order of the GIFs sent before, newest first, as Kotlin's `GifRecents` had it. A GIF is its
//! id here; Kotlin keeps the GIFs and puts them in the order this gives.

/// Where a GIF in the new order comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    /// The GIF at this index of the list before.
    Before(usize),
    /// The GIF just sent, or, after loading, the one sent meanwhile at this index.
    Sent(usize),
}

/// How many GIFs are kept.
pub const CAPACITY: usize = 30;

/// The list after the GIF with `id` was sent, given the `ids` before: it goes first, once, and
/// the oldest beyond `capacity` go.
pub fn add<S: AsRef<[u16]>>(ids: &[S], id: &[u16], capacity: usize) -> Vec<From> {
    let kept = (ids.iter().enumerate())
        .filter(|(_, it)| it.as_ref() != id)
        .map(|(index, _)| From::Before(index));
    let mut order: Vec<From> = std::iter::once(From::Sent(0)).chain(kept).collect();
    order.truncate(capacity);
    order
}

/// The list once the `saved` ids are read, keeping those `sent` meanwhile (newest first): the
/// first `capacity` saved, then each sent, oldest first, added as [`add`] adds it.
pub fn load<S: AsRef<[u16]>>(saved: &[S], sent: &[S], capacity: usize) -> Vec<From> {
    let mut order: Vec<From> = (0..saved.len().min(capacity)).map(From::Before).collect();
    for index in (0..sent.len()).rev() {
        let id = sent[index].as_ref();
        let ids: Vec<&[u16]> = order.iter().map(|&it| id_of(it, saved, sent)).collect();
        order = add(&ids, id, capacity)
            .into_iter()
            .map(|it| match it {
                From::Before(at) => order[at],
                From::Sent(_) => From::Sent(index),
            })
            .collect();
    }
    order
}

fn id_of<'a, S: AsRef<[u16]>>(from: From, saved: &'a [S], sent: &'a [S]) -> &'a [u16] {
    match from {
        From::Before(at) => saved[at].as_ref(),
        From::Sent(at) => sent[at].as_ref(),
    }
}

#[cfg(test)]
mod tests;
