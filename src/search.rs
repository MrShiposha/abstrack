use std::ops::Index;

pub fn search<'b, Buffer, Data>(
    buffer: &'b Buffer, 
    mut left: usize, 
    mut right: usize,
    data: &Data,
) -> (usize, &'b Data)
where 
    Buffer: Index<usize, Output=Data>,
    Data: Ord
{
    while left < right {
        let middle = (left + right) / 2;
        if *data < buffer[middle] {
            right = middle;
        } else {
            left = middle + 1;
        }
    }

    let result = left - 1;
    (result, &buffer[result])
}

#[cfg(test)]
mod test {
    use crate::search::*;

    #[test]
    fn test_search() {
        let v = vec![1, 20, 300, 4000, 50000];

        assert_eq!(search(&v, 0, 5, &1), (0, &1));
        assert_eq!(search(&v, 0, 5, &10), (0, &1));
        assert_eq!(search(&v, 0, 5, &19), (0, &1));
        assert_eq!(search(&v, 0, 5, &20), (1, &20));
        assert_eq!(search(&v, 0, 5, &200), (1, &20));
        assert_eq!(search(&v, 0, 5, &299), (1, &20));
        assert_eq!(search(&v, 0, 5, &300), (2, &300));
        assert_eq!(search(&v, 0, 5, &3000), (2, &300));
        assert_eq!(search(&v, 0, 5, &3999), (2, &300));
        assert_eq!(search(&v, 0, 5, &4000), (3, &4000));
        assert_eq!(search(&v, 0, 5, &40000), (3, &4000));
        assert_eq!(search(&v, 0, 5, &49999), (3, &4000));
        assert_eq!(search(&v, 0, 5, &50000), (4, &50000));
        assert_eq!(search(&v, 0, 5, &500000), (4, &50000));
    }
}