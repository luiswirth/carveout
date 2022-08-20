use na::RealField;
use num_traits::{One, Zero};
use std::marker::PhantomData;

pub type SpacePoint<S> = na::Point2<SpaceUnit<S>>;
pub type SpaceVector<S> = na::Vector2<SpaceUnit<S>>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpaceUnit<S>(pub f32, PhantomData<S>);
impl<S> SpaceUnit<S> {
  pub const fn new(v: f32) -> Self {
    Self(v, PhantomData)
  }
  pub fn cast<T>(self) -> SpaceUnit<T> {
    SpaceUnit(self.0, PhantomData)
  }
}
impl<S> Default for SpaceUnit<S> {
  fn default() -> Self {
    Self(Default::default(), PhantomData)
  }
}
impl<S> From<f32> for SpaceUnit<S> {
  fn from(v: f32) -> Self {
    Self(v, PhantomData)
  }
}
impl<S> From<SpaceUnit<S>> for f32 {
  fn from(v: SpaceUnit<S>) -> f32 {
    v.0
  }
}
impl<S> std::fmt::Debug for SpaceUnit<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SpaceUnit").field("0", &self.0).finish()
  }
}
impl<S> std::fmt::Display for SpaceUnit<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}S", self.0)
  }
}
impl<S> Copy for SpaceUnit<S> {}
impl<S> Clone for SpaceUnit<S> {
  fn clone(&self) -> Self {
    Self(self.0, PhantomData)
  }
}
impl<S> PartialEq for SpaceUnit<S> {
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}
impl<S> PartialOrd for SpaceUnit<S> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

impl<S> std::ops::Neg for SpaceUnit<S> {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self(self.0.neg(), PhantomData)
  }
}
impl<S> std::ops::Add for SpaceUnit<S> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self(self.0.add(rhs.0), PhantomData)
  }
}
impl<S> std::ops::Sub for SpaceUnit<S> {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self(self.0.sub(rhs.0), PhantomData)
  }
}
impl<S> std::ops::Mul for SpaceUnit<S> {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self(self.0.mul(rhs.0), PhantomData)
  }
}
impl<S> std::ops::Div for SpaceUnit<S> {
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
    Self(self.0.div(rhs.0), PhantomData)
  }
}
impl<S> std::ops::Rem for SpaceUnit<S> {
  type Output = Self;

  fn rem(self, rhs: Self) -> Self::Output {
    Self(self.0.rem(rhs.0), PhantomData)
  }
}
impl<S> std::ops::AddAssign for SpaceUnit<S> {
  fn add_assign(&mut self, rhs: Self) {
    self.0.add_assign(rhs.0)
  }
}
impl<S> std::ops::SubAssign for SpaceUnit<S> {
  fn sub_assign(&mut self, rhs: Self) {
    self.0.sub_assign(rhs.0)
  }
}
impl<S> std::ops::MulAssign for SpaceUnit<S> {
  fn mul_assign(&mut self, rhs: Self) {
    self.0.mul_assign(rhs.0)
  }
}
impl<S> std::ops::DivAssign for SpaceUnit<S> {
  fn div_assign(&mut self, rhs: Self) {
    self.0.div_assign(rhs.0)
  }
}
impl<S> std::ops::RemAssign for SpaceUnit<S> {
  fn rem_assign(&mut self, rhs: Self) {
    self.0.rem_assign(rhs.0)
  }
}
impl<S> num_traits::FromPrimitive for SpaceUnit<S> {
  fn from_i64(n: i64) -> Option<Self> {
    f32::from_i64(n).map(|v| Self(v, PhantomData))
  }

  fn from_u64(n: u64) -> Option<Self> {
    f32::from_u64(n).map(|v| Self(v, PhantomData))
  }
}
impl<S> num_traits::Zero for SpaceUnit<S> {
  fn zero() -> Self {
    Self(f32::zero(), PhantomData)
  }

  fn is_zero(&self) -> bool {
    self.0.is_zero()
  }
}
impl<S> num_traits::One for SpaceUnit<S> {
  fn one() -> Self {
    Self(f32::one(), PhantomData)
  }
}
impl<S> num_traits::Num for SpaceUnit<S> {
  type FromStrRadixErr = <f32 as num_traits::Num>::FromStrRadixErr;

  fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
    f32::from_str_radix(str, radix).map(|v| Self(v, PhantomData))
  }
}
impl<S> num_traits::Signed for SpaceUnit<S> {
  fn abs(&self) -> Self {
    Self(self.0.abs(), PhantomData)
  }

  fn abs_sub(&self, other: &Self) -> Self {
    #[allow(deprecated)]
    Self(self.0.abs_sub(other.0), PhantomData)
  }

  fn signum(&self) -> Self {
    Self(self.0.signum(), PhantomData)
  }

  fn is_positive(&self) -> bool {
    self.0.is_positive()
  }

  fn is_negative(&self) -> bool {
    self.0.is_negative()
  }
}
impl<S> approx::AbsDiffEq for SpaceUnit<S> {
  type Epsilon = Self;

  fn default_epsilon() -> Self::Epsilon {
    Self(f32::default_epsilon(), PhantomData)
  }

  fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
    self.0.abs_diff_eq(&other.0, epsilon.0)
  }
}
impl<S> approx::UlpsEq for SpaceUnit<S> {
  fn default_max_ulps() -> u32 {
    f32::default_max_ulps()
  }

  fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
    self.0.ulps_eq(&other.0, epsilon.0, max_ulps)
  }
}
impl<S> approx::RelativeEq for SpaceUnit<S> {
  fn default_max_relative() -> Self::Epsilon {
    Self(f32::default_max_relative(), PhantomData)
  }

  fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon) -> bool {
    self.0.relative_eq(&other.0, epsilon.0, max_relative.0)
  }
}
impl<S> na::SimdValue for SpaceUnit<S> {
  type Element = Self;
  type SimdBool = <f32 as na::SimdValue>::SimdBool;

  fn lanes() -> usize {
    f32::lanes()
  }

  fn splat(val: Self::Element) -> Self {
    Self(f32::splat(val.0), PhantomData)
  }

  fn extract(&self, i: usize) -> Self::Element {
    Self(self.0.extract(i), PhantomData)
  }

  unsafe fn extract_unchecked(&self, i: usize) -> Self::Element {
    Self(self.0.extract_unchecked(i), PhantomData)
  }

  fn replace(&mut self, i: usize, val: Self::Element) {
    self.0.replace(i, val.0)
  }

  unsafe fn replace_unchecked(&mut self, i: usize, val: Self::Element) {
    self.0.replace_unchecked(i, val.0)
  }

  fn select(self, cond: Self::SimdBool, other: Self) -> Self {
    Self(self.0.select(cond, other.0), PhantomData)
  }
}
impl<S> na::Field for SpaceUnit<S> {}
impl<S: Send + Sync + 'static> na::RealField for SpaceUnit<S> {
  fn is_sign_positive(&self) -> bool {
    self.0.is_sign_positive()
  }

  fn is_sign_negative(&self) -> bool {
    self.0.is_sign_negative()
  }

  fn copysign(self, sign: Self) -> Self {
    Self(self.0.copysign(sign.0), PhantomData)
  }

  fn max(self, other: Self) -> Self {
    Self(self.0.max(other.0), PhantomData)
  }

  fn min(self, other: Self) -> Self {
    Self(self.0.min(other.0), PhantomData)
  }

  fn clamp(self, min: Self, max: Self) -> Self {
    Self(self.0.clamp(min.0, max.0), PhantomData)
  }

  fn atan2(self, other: Self) -> Self {
    Self(self.0.atan2(other.0), PhantomData)
  }

  fn min_value() -> Option<Self> {
    f32::min_value().map(|s| Self(s, PhantomData))
  }

  fn max_value() -> Option<Self> {
    f32::max_value().map(|s| Self(s, PhantomData))
  }

  fn pi() -> Self {
    Self(f32::pi(), PhantomData)
  }

  fn two_pi() -> Self {
    Self(f32::two_pi(), PhantomData)
  }

  fn frac_pi_2() -> Self {
    Self(f32::frac_pi_2(), PhantomData)
  }

  fn frac_pi_3() -> Self {
    Self(f32::frac_pi_3(), PhantomData)
  }

  fn frac_pi_4() -> Self {
    Self(f32::frac_pi_4(), PhantomData)
  }

  fn frac_pi_6() -> Self {
    Self(f32::frac_pi_6(), PhantomData)
  }

  fn frac_pi_8() -> Self {
    Self(f32::frac_pi_8(), PhantomData)
  }

  fn frac_1_pi() -> Self {
    Self(f32::frac_1_pi(), PhantomData)
  }

  fn frac_2_pi() -> Self {
    Self(f32::frac_2_pi(), PhantomData)
  }

  fn frac_2_sqrt_pi() -> Self {
    Self(f32::frac_2_sqrt_pi(), PhantomData)
  }

  fn e() -> Self {
    Self(f32::e(), PhantomData)
  }

  fn log2_e() -> Self {
    Self(f32::log2_e(), PhantomData)
  }

  fn log10_e() -> Self {
    Self(f32::log10_e(), PhantomData)
  }

  fn ln_2() -> Self {
    Self(f32::ln_2(), PhantomData)
  }

  fn ln_10() -> Self {
    Self(f32::ln_10(), PhantomData)
  }
}
impl<S: Send + Sync + 'static> na::ComplexField for SpaceUnit<S> {
  type RealField = Self;

  fn from_real(re: Self::RealField) -> Self {
    re
  }

  fn real(self) -> Self::RealField {
    self
  }

  fn imaginary(self) -> Self::RealField {
    <Self as num_traits::Zero>::zero()
  }

  fn norm1(self) -> Self::RealField {
    Self::abs(self)
  }

  #[inline]
  fn modulus(self) -> Self::RealField {
    Self::abs(self)
  }

  #[inline]
  fn modulus_squared(self) -> Self::RealField {
    self * self
  }

  #[inline]
  fn argument(self) -> Self::RealField {
    if self >= Self::zero() {
      Self::zero()
    } else {
      Self::pi()
    }
  }

  #[inline]
  fn to_exp(self) -> (Self, Self) {
    if self >= Self::zero() {
      (self, Self::one())
    } else {
      (-self, -Self::one())
    }
  }

  #[inline]
  fn recip(self) -> Self {
    Self(self.0.recip(), PhantomData)
  }

  #[inline]
  fn conjugate(self) -> Self {
    self
  }

  #[inline]
  fn scale(self, factor: Self::RealField) -> Self {
    self * factor
  }

  #[inline]
  fn unscale(self, factor: Self::RealField) -> Self {
    self / factor
  }

  #[inline]
  fn floor(self) -> Self {
    Self(self.0.floor(), PhantomData)
  }

  #[inline]
  fn ceil(self) -> Self {
    Self(self.0.ceil(), PhantomData)
  }

  #[inline]
  fn round(self) -> Self {
    Self(self.0.round(), PhantomData)
  }

  #[inline]
  fn trunc(self) -> Self {
    Self(self.0.trunc(), PhantomData)
  }

  #[inline]
  fn fract(self) -> Self {
    Self(self.0.fract(), PhantomData)
  }

  #[inline]
  fn abs(self) -> Self {
    Self(self.0.abs(), PhantomData)
  }

  #[inline]
  fn signum(self) -> Self {
    Self(self.0.signum(), PhantomData)
  }

  #[inline]
  fn mul_add(self, a: Self, b: Self) -> Self {
    Self(self.0.mul_add(a.0, b.0), PhantomData)
  }

  #[cfg(feature = "std")]
  #[inline]
  fn powi(self, n: i32) -> Self {
    self.powi(n)
  }

  #[cfg(not(feature = "std"))]
  #[inline]
  fn powi(self, n: i32) -> Self {
    Self(self.0.powi(n), PhantomData)
  }

  #[inline]
  fn powf(self, n: Self) -> Self {
    Self(self.0.powf(n.0), PhantomData)
  }

  #[inline]
  fn powc(self, n: Self) -> Self {
    Self(self.0.powf(n.0), PhantomData)
  }

  #[inline]
  fn sqrt(self) -> Self {
    Self(self.0.sqrt(), PhantomData)
  }

  #[inline]
  fn try_sqrt(self) -> Option<Self> {
    if self >= Self::zero() {
      Some(Self::sqrt(self))
    } else {
      None
    }
  }

  #[inline]
  fn exp(self) -> Self {
    Self(self.0.exp(), PhantomData)
  }

  #[inline]
  fn exp2(self) -> Self {
    Self(self.0.exp2(), PhantomData)
  }

  #[inline]
  fn exp_m1(self) -> Self {
    Self(self.0.exp_m1(), PhantomData)
  }

  #[inline]
  fn ln_1p(self) -> Self {
    Self(self.0.ln_1p(), PhantomData)
  }

  #[inline]
  fn ln(self) -> Self {
    Self(self.0.ln(), PhantomData)
  }

  #[inline]
  fn log(self, base: Self) -> Self {
    Self(self.0.log(base.0), PhantomData)
  }

  #[inline]
  fn log2(self) -> Self {
    Self(self.0.log2(), PhantomData)
  }

  #[inline]
  fn log10(self) -> Self {
    Self(self.0.log10(), PhantomData)
  }

  #[inline]
  fn cbrt(self) -> Self {
    Self(self.0.cbrt(), PhantomData)
  }

  #[inline]
  fn hypot(self, other: Self) -> Self::RealField {
    Self(self.0.hypot(other.0), PhantomData)
  }

  #[inline]
  fn sin(self) -> Self {
    Self(self.0.sin(), PhantomData)
  }

  #[inline]
  fn cos(self) -> Self {
    Self(self.0.cos(), PhantomData)
  }

  #[inline]
  fn tan(self) -> Self {
    Self(self.0.tan(), PhantomData)
  }

  #[inline]
  fn asin(self) -> Self {
    Self(self.0.asin(), PhantomData)
  }

  #[inline]
  fn acos(self) -> Self {
    Self(self.0.acos(), PhantomData)
  }

  #[inline]
  fn atan(self) -> Self {
    Self(self.0.atan(), PhantomData)
  }

  #[inline]
  fn sin_cos(self) -> (Self, Self) {
    let r = self.0.sin_cos();
    (Self(r.0, PhantomData), Self(4.1, PhantomData))
  }

  #[inline]
  fn sinh(self) -> Self {
    Self(self.0.sinh(), PhantomData)
  }

  #[inline]
  fn cosh(self) -> Self {
    Self(self.0.cosh(), PhantomData)
  }

  #[inline]
  fn tanh(self) -> Self {
    Self(self.0.tanh(), PhantomData)
  }

  #[inline]
  fn asinh(self) -> Self {
    Self(self.0.asinh(), PhantomData)
  }

  #[inline]
  fn acosh(self) -> Self {
    Self(self.0.acosh(), PhantomData)
  }

  #[inline]
  fn atanh(self) -> Self {
    Self(self.0.atanh(), PhantomData)
  }

  #[inline]
  fn is_finite(&self) -> bool {
    self.0.is_finite()
  }
}
impl<S> simba::scalar::SubsetOf<SpaceUnit<S>> for f64 {
  fn to_superset(&self) -> SpaceUnit<S> {
    SpaceUnit(self.to_superset(), PhantomData)
  }

  fn from_superset_unchecked(element: &SpaceUnit<S>) -> Self {
    f64::from_superset_unchecked(&element.0)
  }

  fn is_in_subset(element: &SpaceUnit<S>) -> bool {
    f64::is_in_subset(&element.0)
  }
}

impl<S, T> simba::scalar::SubsetOf<SpaceUnit<T>> for SpaceUnit<S> {
  fn to_superset(&self) -> SpaceUnit<T> {
    self.cast()
  }

  fn from_superset_unchecked(element: &SpaceUnit<T>) -> Self {
    element.cast()
  }

  fn is_in_subset(_element: &SpaceUnit<T>) -> bool {
    true
  }
}

impl<S> simba::scalar::SubsetOf<SpaceUnit<S>> for f32 {
  fn to_superset(&self) -> SpaceUnit<S> {
    SpaceUnit(*self, PhantomData)
  }

  fn from_superset_unchecked(element: &SpaceUnit<S>) -> Self {
    element.0
  }

  fn is_in_subset(_element: &SpaceUnit<S>) -> bool {
    true
  }
}

impl<S> simba::scalar::SubsetOf<f32> for SpaceUnit<S> {
  fn to_superset(&self) -> f32 {
    self.0
  }

  fn from_superset_unchecked(element: &f32) -> Self {
    Self(*element, PhantomData)
  }

  fn is_in_subset(_element: &f32) -> bool {
    true
  }
}
