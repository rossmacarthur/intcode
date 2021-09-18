//! Try unpack a vector into a tuple.

pub trait TryUnpack<T>: Sized {
    fn try_unpack(vec: Vec<T>) -> Result<Self, (usize, usize)>;
}

impl<T> TryUnpack<T> for () {
    fn try_unpack(vec: Vec<T>) -> Result<(), (usize, usize)> {
        if !vec.is_empty() {
            return Err((0, vec.len()));
        }
        Ok(())
    }
}

impl<T> TryUnpack<T> for (T,) {
    fn try_unpack(vec: Vec<T>) -> Result<(T,), (usize, usize)> {
        if vec.len() != 1 {
            return Err((1, vec.len()));
        }
        let mut it = vec.into_iter();
        Ok((it.next().unwrap(),))
    }
}

impl<T> TryUnpack<T> for (T, T) {
    fn try_unpack(vec: Vec<T>) -> Result<(T, T), (usize, usize)> {
        if vec.len() != 2 {
            return Err((2, vec.len()));
        }
        let mut it = vec.into_iter();
        Ok((it.next().unwrap(), it.next().unwrap()))
    }
}

impl<T> TryUnpack<T> for (T, T, T) {
    fn try_unpack(vec: Vec<T>) -> Result<(T, T, T), (usize, usize)> {
        if vec.len() != 3 {
            return Err((3, vec.len()));
        }
        let mut it = vec.into_iter();
        Ok((it.next().unwrap(), it.next().unwrap(), it.next().unwrap()))
    }
}
