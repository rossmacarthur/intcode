//! Try unpack a vector into a tuple.

pub trait TryUnpack<T> {
    fn try_unpack(self) -> Result<T, (usize, usize)>;
}

impl<U> TryUnpack<()> for Vec<U> {
    fn try_unpack(self) -> Result<(), (usize, usize)> {
        if !self.is_empty() {
            return Err((0, self.len()));
        }
        Ok(())
    }
}

impl<U> TryUnpack<(U,)> for Vec<U> {
    fn try_unpack(self) -> Result<(U,), (usize, usize)> {
        if self.len() != 1 {
            return Err((1, self.len()));
        }
        let mut it = self.into_iter();
        Ok((it.next().unwrap(),))
    }
}

impl<U> TryUnpack<(U, U)> for Vec<U> {
    fn try_unpack(self) -> Result<(U, U), (usize, usize)> {
        if self.len() != 2 {
            return Err((2, self.len()));
        }
        let mut it = self.into_iter();
        Ok((it.next().unwrap(), it.next().unwrap()))
    }
}

impl<U> TryUnpack<(U, U, U)> for Vec<U> {
    fn try_unpack(self) -> Result<(U, U, U), (usize, usize)> {
        if self.len() != 3 {
            return Err((3, self.len()));
        }
        let mut it = self.into_iter();
        Ok((it.next().unwrap(), it.next().unwrap(), it.next().unwrap()))
    }
}
