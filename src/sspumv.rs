mod make_spline;
mod evaluate_spline;
mod validate_data;

use ndarray::{
    NdFloat,
    Dimension,
    Axis,
    AsArray,
    Array,
    Array2,
    ArrayView,
    ArrayView1,
    ArrayView2,
};


use crate::Result;


/// N-dimensional spline representation
#[derive(Debug)]
pub struct NdSpline<T>
    where T: NdFloat
{
    ndim: usize,
    order: usize,
    pieces: usize,
    coeffs: Array2<T>,
}


impl<T> NdSpline<T>
    where T: NdFloat
{
    pub fn ndim(&self) -> usize { self.ndim }

    pub fn order(&self) -> usize { self.order }

    pub fn pieces(&self) -> usize { self.pieces }

    pub fn coeffs(&self) -> ArrayView2<'_, T> { self.coeffs.view() }
}


/// N-dimensional (univariate/multivariate) smoothing spline calculator/evaluator
pub struct CubicSmoothingSpline<'a, T, D>
    where T: NdFloat, D: Dimension
{
    x: ArrayView1<'a, T>,
    y: ArrayView<'a, T, D>,

    axis: Option<Axis>,

    weights: Option<ArrayView1<'a, T>>,
    smooth: Option<T>,

    spline: Option<NdSpline<T>>
}


impl<'a, T, D> CubicSmoothingSpline<'a, T, D>
    where T: NdFloat + Default, D: Dimension
{
    pub fn new<V, Nd>(x: V, y: Nd) -> Self
        where V: AsArray<'a, T>,
              Nd: AsArray<'a, T, D>
    {
        CubicSmoothingSpline {
            x: x.into(),
            y: y.into(),
            axis: None,
            weights: None,
            smooth: None,
            spline: None,
        }
    }

    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.invalidate();
        self.axis = Some(axis);
        self
    }

    pub fn with_weights<V>(mut self, weights: V) -> Self
        where V: AsArray<'a, T>
    {
        self.invalidate();
        self.weights = Some(weights.into());
        self
    }

    pub fn with_smooth(mut self, smooth: T) -> Self {
        self.invalidate();
        self.smooth = Some(smooth);
        self
    }

    pub fn make(mut self) -> Result<Self> {
        self.make_validate_data()?;
        self.make_spline()?;
        Ok(self)
    }

    pub fn evaluate<V>(&self, xi: V) -> Result<Array<T, D>>
        where V: AsArray<'a, T>
    {
        let xi = xi.into();

        self.evaluate_validate_data(&xi)?;
        let ys = self.evaluate_spline(xi);
        Ok(ys)
    }

    pub fn smooth(&self) -> Option<T> {
        self.smooth
    }

    pub fn spline(&self) -> Option<&NdSpline<T>> {
        match &self.spline {
            Some(spline) => Some(spline),
            None => None,
        }
    }

    fn invalidate(&mut self) {
        self.spline = None;
    }
}
