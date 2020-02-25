use num_traits::NumOps;

use ndarray::{
    ScalarOperand,
    Dimension,
    AsArray,
    Array,
    Axis,
    Slice,
};


pub fn diff<'a, T: 'a, D, V>(data: V, axis: Option<Axis>) -> Array<T, D>
    where T: NumOps + ScalarOperand, D: Dimension, V: AsArray<'a, T, D>
{
    let data_view = data.into();
    let axis = axis.unwrap_or(Axis(data_view.ndim() - 1));

    let head = data_view.slice_axis(axis, Slice::from(..-1));
    let tail = data_view.slice_axis(axis, Slice::from(1..));

    &tail - &head
}


#[cfg(test)]
mod tests {
    use ndarray::{array, Axis};
    use crate::arrayfuncs::diff;

    #[test]
    fn test_diff_1d() {
        let a = array![1, 2, 3, 4, 5];

        assert_eq!(diff(&a, None),
                   array![1, 1, 1, 1]);

        assert_eq!(diff(&a, Some(Axis(0))),
                   array![1, 1, 1, 1]);
    }

    #[test]
    fn test_diff_2d() {
        let a = array![[1., 2., 3., 4.], [1., 2., 3., 4.]];

        assert_eq!(diff(&a, None),
                   array![[1., 1., 1.], [1., 1., 1.]]);

        assert_eq!(diff(&a, Some(Axis(0))),
                   array![[0., 0., 0., 0.]]);

        assert_eq!(diff(&a, Some(Axis(1))),
                   array![[1., 1., 1.], [1., 1., 1.]]);
    }

    #[test]
    fn test_diff_3d() {
        let a = array![[[1., 2., 3.], [1., 2., 3.]], [[1., 2., 3.], [1., 2., 3.]]];

        assert_eq!(diff(&a, None),
                   array![[[1., 1.], [1., 1.]], [[1., 1.], [1., 1.]]]);

        assert_eq!(diff(&a, Some(Axis(0))),
                   array![[[0., 0., 0.], [0., 0., 0.]]]);

        assert_eq!(diff(&a, Some(Axis(1))),
                   array![[[0., 0., 0.]], [[0., 0., 0.]]]);

        assert_eq!(diff(&a, Some(Axis(2))),
                   array![[[1., 1.], [1., 1.]], [[1., 1.], [1., 1.]]]);
    }
}
