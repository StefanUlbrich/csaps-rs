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

use almost::AlmostEqual;

use crate::Result;

mod validate_data;
mod make_spline;
mod evaluate_spline;


/// N-dimensional (univariate/multivariate) spline PP-form representation
///
/// `NdSpline` represents n-d splines as the set of attributes and
/// array of picewise-polinomial coefficients.
#[derive(Debug)]
pub struct NdSpline<'a, T: NdFloat>
{
    /// The spline dimensionality
    ndim: usize,

    /// The spline order, 4 for cubic spline for example
    order: usize,

    /// The number of pieces of the spline
    pieces: usize,

    /// The breaks (data sites) which have been used for computing spline
    breaks: ArrayView1<'a, T>,

    /// The NxM array of spline coefficients where N is `ndim` and M is row of pieces of coefficients
    coeffs: Array2<T>,
}


impl<'a, T> NdSpline<'a, T>
    where T: NdFloat + AlmostEqual
{
    /// Creates `NdSpline` struct from given `order`, `breaks` and `coeffs`
    ///
    /// Arguments:
    ///
    /// - `order` -- The spline order, 4 for cubic spline for example
    /// - `breaks` -- The breaks (data sites) which have been used for computing spline
    /// - `coeffs` -- The NxM array of spline coefficients where N is `ndim` and M is row of pieces of coefficients
    ///
    /// Note:
    ///
    /// - `NdSpline` struct should not be created directly in most cases.
    ///
    pub fn new(order: usize, breaks: ArrayView1<'a, T>, coeffs: Array2<T>) -> NdSpline<'a, T> {
        let c_shape = coeffs.shape();
        let ndim = c_shape[0];
        let pieces = c_shape[1] / order;

        NdSpline {
            ndim,
            order,
            pieces,
            breaks,
            coeffs,
        }
    }

    /// Returns the spline dimensionality
    pub fn ndim(&self) -> usize { self.ndim }

    /// Returns the spline order
    pub fn order(&self) -> usize { self.order }

    /// Returns the number of pieces of the spline
    pub fn pieces(&self) -> usize { self.pieces }

    /// Returns the view to the breaks array
    pub fn breaks(&self) -> ArrayView1<'_, T> { self.breaks.view() }

    /// Returns the view to the spline coefficients array
    pub fn coeffs(&self) -> ArrayView2<'_, T> { self.coeffs.view() }

    /// Evaluates the spline on the given data sites
    pub fn evaluate<X>(&self, xi: X) -> Array2<T>
        where X: AsArray<'a, T>
    {
        self.evaluate_spline(xi.into())
    }
}


/// N-dimensional (univariate/multivariate) smoothing spline calculator/evaluator
///
/// The struct represents n-d smoothing cubic spline and allows you to make and evaluate the
/// splines for given data.
///
/// `CubicSmoothingSpline` struct is parametrized by data type (`f64` or `f32`)
/// and data dimension.
///
/// The methods API of `CubicSmoothingSpline` is implemented as "builder" pattern or in other
/// words as chained API:
///
/// ```
/// use csaps::CubicSmoothingSpline;
///
/// let x = vec![1.0, 2.0, 3.0, 4.0];
/// let y = vec![0.5, 1.2, 3.4, 2.5];
///
/// let ys = CubicSmoothingSpline::new(&x, &y)
///     .make().unwrap()
///     .evaluate(&x).unwrap();
/// ```
///
/// ```
/// use ndarray::array;
/// use csaps::CubicSmoothingSpline;
///
/// let x = array![1.0, 2.0, 3.0, 4.0];
/// let y = array![0.5, 1.2, 3.4, 2.5];
/// let w = array![1.0, 0.7, 0.5, 1.0];
/// let smooth = 0.85;
///
/// let s = CubicSmoothingSpline::new(&x, &y)
///     .with_weights(&w)
///     .with_smooth(smooth)
///     .make().unwrap();
///
/// let xi = array![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
/// let yi = s.evaluate(&xi).unwrap();
/// ```
///
pub struct CubicSmoothingSpline<'a, T, D>
    where T: NdFloat, D: Dimension
{
    /// X data sites (also breaks)
    x: ArrayView1<'a, T>,

    /// Y data values
    y: ArrayView<'a, T, D>,

    /// The axis parameter defines axis of Y data for spline computing
    axis: Option<Axis>,

    /// The optional data weights
    weights: Option<ArrayView1<'a, T>>,

    /// The optional smoothing parameter
    smooth: Option<T>,

    /// `NdSpline` struct with computed spline
    spline: Option<NdSpline<'a, T>>
}


impl<'a, T, D> CubicSmoothingSpline<'a, T, D>
    where T: NdFloat + Default + AlmostEqual, D: Dimension
{
    /// Creates `CubicSmoothingSpline` struct from the given X and Y data
    ///
    /// Arguments:
    ///
    /// - `x` -- the X-data sites 1-d array-like. Must strictly increasing: x1 < x2 < x3 < ... < xN
    /// - `y` -- The Y-data values n-d array-like. `ndim` can be from 1 to N. The splines will be computed for
    ///   all data by given axis. By default the axis parameter is equal to the last axis of Y data.
    ///   For example, for 1-d axis is equal to 0, for 2-d axis is equal to 1, for 3-d axis is
    ///   equal to 2 and etc.
    ///
    pub fn new<X, Y>(x: X, y: Y) -> Self
        where X: AsArray<'a, T>,
              Y: AsArray<'a, T, D>
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

    /// Sets the axis parameter
    ///
    /// The Y-data axis. Axis along which Y-data is assumed to be varying.
    /// In other words, the axis parameter specifies Y-data axis for computing spline.
    ///
    /// `y.shape()[axis]` must be equal to `x.len()`
    ///
    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.invalidate();
        self.axis = Some(axis);
        self
    }

    /// Sets the data weights
    ///
    /// `weights.len()` must be equal to `x.len()`
    ///
    pub fn with_weights<W>(mut self, weights: W) -> Self
        where W: AsArray<'a, T>
    {
        self.invalidate();
        self.weights = Some(weights.into());
        self
    }

    /// Sets the smoothing parameter
    ///
    /// The smoothing parameter should be in range `[0, 1]`,
    /// where bounds are:
    ///
    ///  - 0: The smoothing spline is the least-squares straight line fit to the data
    ///  - 1: The cubic spline interpolant with natural boundary condition
    ///
    pub fn with_smooth(mut self, smooth: T) -> Self {
        self.invalidate();
        self.smooth = Some(smooth);
        self
    }

    /// Makes (computes) spline for given data and parameters
    ///
    /// # Errors
    ///
    /// - If the data or parameters are invalid
    /// - If reshaping Y data to 2-d view has failed
    ///
    pub fn make(mut self) -> Result<Self> {
        self.make_validate_data()?;
        self.make_spline()?;
        Ok(self)
    }

    /// Evaluates the computed spline on the given data sites
    ///
    /// # Errors
    ///
    /// - If the `xi` data is invalid
    /// - If the spline yet has not been computed
    ///
    pub fn evaluate<X>(&self, xi: X) -> Result<Array<T, D>>
        where X: AsArray<'a, T>
    {
        let xi = xi.into();
        self.evaluate_validate_data(xi)?;

        let yi = self.evaluate_spline(xi)?;
        Ok(yi)
    }

    /// Returns the smoothing parameter
    ///
    /// # Panics
    ///
    /// - If the smoothing parameter has not been set or computed
    ///
    pub fn smooth(&self) -> T {
        self.smooth.expect("The smoothing parameter yet has not been set or computed.")
    }

    /// Returns `NdSpline` struct with data of computed spline
    ///
    /// # Panics
    ///
    /// - If the spline has not been computed
    ///
    pub fn spline(&self) -> &NdSpline<'a, T> {
        self.spline.as_ref().expect("The spline yet has not been computed.")
    }

    /// Invalidate computed spline
    fn invalidate(&mut self) {
        self.spline = None;
    }
}
