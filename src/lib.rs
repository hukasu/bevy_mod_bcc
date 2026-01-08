//! # Binary curve collection
//!
//! Created by [Cem Yuksel](https://www.cemyuksel.com/research/yarnmodels/) for defining yarn-level cloth models.

#[cfg(feature = "bevy")]
pub mod plugin;
pub mod reader;

use std::{
    error::Error,
    fmt::{Debug, Display},
};
#[cfg(feature = "bevy")]
use {bevy_asset::prelude::*, bevy_reflect::prelude::*};

/// A collection of curves.
///
/// Curves can only be represented by Catmull-Rom curves with uniform parameterization in 3d
/// space with [`i32`] and [`f32`] precisions.
#[cfg_attr(feature = "bevy", derive(TypePath, Asset))]
#[derive(Clone)]
pub struct BinaryCurveCollection {
    /// Header of the collection
    header: BinaryCurveCollectionHeader,
    /// Flag if curve N is looping. This list has length
    /// [`number_of_curves`](BinaryCurveCollectionHeader::number_of_curves).
    looping: Box<[bool]>,
    /// Index of the first control point. This list has length
    /// [`number_of_curves`](BinaryCurveCollectionHeader::number_of_curves) + 1.
    first_control_points: Box<[usize]>,
    /// Control points
    control_points: Box<[f32]>,
}

impl BinaryCurveCollection {
    /// Get the header of the [`BinaryCurveCollection`]
    pub fn header(&self) -> &BinaryCurveCollectionHeader {
        &self.header
    }

    /// Check if the Nth curve is looping. This accept values for N from
    /// 0 through [`number_of_curves`](BinaryCurveCollectionHeader::number_of_curves) exclusive.
    pub fn looping(&self, n: usize) -> Option<bool> {
        self.looping.get(n).copied()
    }

    /// Get the first control point of the Nth curve. This accept values for N from
    /// 0 through [`number_of_curves`](BinaryCurveCollectionHeader::number_of_curves) inclusive.
    pub fn first_control_point(&self, n: usize) -> Option<usize> {
        self.first_control_points.get(n).copied()
    }

    /// Get the control points of the [`BinaryCurveCollection`].
    ///
    /// The slice returned by this method is flattened, which means that you
    /// will need to traverse it using something like [`Windows`](std::slice::Windows)
    /// passing [`BinaryCurveCollectionHeader::dimensions()`] as parameter, or converting it into
    /// the appropriate vector of the appropriate dimension on your math library of choice.
    ///
    /// # Example
    /// ```
    /// # use bevy_mod_bcc::{BinaryCurveCollection, BinaryCurveCollectionHeader};
    /// # const binary_curve_collection: &[u8] = &[
    /// #   b'B', b'C', b'C', 0x44, b'C', b'0', 3, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    /// #   1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /// #   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /// #   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    /// #   1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x3f, 0, 0, 0, 0x40
    /// # ];
    /// # let bcc = BinaryCurveCollection::parse(binary_curve_collection).unwrap();
    /// let first_control_point = bcc.first_control_point(0).unwrap();
    /// let control_points = bcc.control_points();
    /// let x = control_points[first_control_point];
    /// let y = control_points[first_control_point + 1];
    /// let z = control_points[first_control_point + 2];
    /// # assert_eq!(x, 0.);
    /// # assert_eq!(y, 1.);
    /// # assert_eq!(z, 2.);
    /// ```
    pub fn control_points(&self) -> &[f32] {
        &self.control_points
    }
}

/// Header of a [`BinaryCurveCollection`]
#[derive(Clone, Copy)]
pub struct BinaryCurveCollectionHeader {
    /// Signature of the file. Must be `BCC`.
    signature: [u8; 3],
    /// Precision of the curves. High nible represent integer precision, and must be 4.
    /// Low nible represents float precision, and must be 4.
    precision: u8,
    /// Type of curve
    ///
    /// * `C0`: Catmull-Rom curves with uniform parameterization
    curve: [u8; 2],
    /// Number of dimensions. Should always 3.
    dimensions: u8,
    /// Up direction
    ///
    /// * `1`: Y
    /// * `2`: Z
    up_direction: u8,
    /// Number of curves
    number_of_curves: u64,
    /// Number of control points
    number_of_control_points: u64,
    /// File information in ASCII
    file_information: [u8; 40],
}

impl BinaryCurveCollectionHeader {
    /// Get the dimensions of the curves.
    ///
    /// Should alwasy be 3.
    pub fn dimensions(&self) -> u8 {
        self.dimensions
    }

    /// Get the dimensions of the curves.
    pub fn number_of_curves(&self) -> u64 {
        self.number_of_curves
    }
}

impl Debug for BinaryCurveCollectionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinaryCurveCollectionHeader")
            .field("signature", &String::from_utf8_lossy(&self.signature))
            .field("precision", &format!("{:#02x}", self.precision))
            .field("curve", &String::from_utf8_lossy(&self.curve))
            .field("dimensions", &self.dimensions)
            .field(
                "up_direction",
                if self.up_direction == 1 {
                    &"Y"
                } else if self.up_direction == 2 {
                    &"Z"
                } else {
                    &"Unknown"
                },
            )
            .field("number_of_curves", &self.number_of_curves)
            .field("number_of_control_points", &self.number_of_control_points)
            .field(
                "file_information",
                &String::from_utf8_lossy(&self.file_information).trim_end_matches("\0"),
            )
            .finish()
    }
}

/// Errors that can happen while parsing a [`BinaryCurveCollection`].
#[derive(Debug)]
pub enum BinaryCurveCollectionParserError {
    /// The signature was not `BCC`
    InvalidSignature,
    /// The precision was not 0x44
    InvalidPrecision,
    /// The curve type was not one of:
    ///
    /// * `C0`: Catmull-Rom curve with uniform parameterization
    InvalidCurve,
    /// The up directions was not one of:
    ///
    /// * `1`: Y up
    /// * `2`: Z up
    InvalidUpDirection,
    /// The number of curves does not fit in memory (i.e., larger than [`usize::MAX`])
    TooManyCurves,
    /// The number of control points does not fit in memory (i.e., larger than [`usize::MAX`])
    TooManyControlPoints,
    /// An Io error occurred while reading from the [`std::io::Read`].
    Io(std::io::Error),
}

impl Display for BinaryCurveCollectionParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "BCC file had invalid signature."),
            Self::InvalidPrecision => write!(
                f,
                "BCC files only support 4 byte integers and 4 byte floats."
            ),
            Self::InvalidCurve => write!(
                f,
                "BCC files only support `C0` Catmull-Rom curves with uniform parameterization."
            ),
            Self::InvalidUpDirection => write!(
                f,
                "BCC files only support `1` Y-up or `2` Z-up coordinate systems."
            ),
            Self::TooManyCurves => write!(
                f,
                "BCC files contains more curves than system can hold in memory."
            ),
            Self::TooManyControlPoints => write!(
                f,
                "BCC files contains more control points than system can hold in memory."
            ),
            Self::Io(err) => write!(f, "Io error during parsing of BCC file. {err}."),
        }
    }
}

impl Error for BinaryCurveCollectionParserError {}

impl From<std::io::Error> for BinaryCurveCollectionParserError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
