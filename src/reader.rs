//! Reader for a [`BinaryCurveCollection`].

use std::io::Read;

use bevy_asset::AsyncReadExt;
use futures_io::AsyncRead;
use log::{debug, trace};

use crate::{BinaryCurveCollection, BinaryCurveCollectionHeader, BinaryCurveCollectionParserError};

impl BinaryCurveCollection {
    /// Parses a [`Read`] into a [`BinaryCurveCollection`].
    ///
    /// This method has an `async` counterpart [`BinaryCurveCollection::parse_async`].
    pub fn parse<T: Read>(mut reader: T) -> Result<Self, BinaryCurveCollectionParserError> {
        debug!("Parsing BinaryCurveCollection");
        let mut signature = [0; 3];
        reader.read_exact(&mut signature)?;
        if signature != [b'B', b'C', b'C'] {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut precision = 0;
        reader.read_exact(std::slice::from_mut(&mut precision))?;
        if precision != 0x44 {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut curve = [0; 2];
        reader.read_exact(&mut curve)?;
        if curve != [b'C', b'0'] {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut dimensions = 0;
        reader.read_exact(std::slice::from_mut(&mut dimensions))?;

        let mut up_direction = 0;
        reader.read_exact(std::slice::from_mut(&mut up_direction))?;
        if !(1..=2).contains(&up_direction) {
            return Err(BinaryCurveCollectionParserError::InvalidUpDirection);
        }

        let mut number_of_curves = [0; 8];
        reader.read_exact(&mut number_of_curves)?;
        let number_of_curves = u64::from_le_bytes(number_of_curves);

        let mut number_of_control_points = [0; 8];
        reader.read_exact(&mut number_of_control_points)?;
        let number_of_control_points = u64::from_le_bytes(number_of_control_points);

        let mut file_information = [0; 40];
        reader.read_exact(&mut file_information)?;

        let Ok(size_of_curves) = usize::try_from(number_of_curves) else {
            return Err(BinaryCurveCollectionParserError::TooManyCurves);
        };
        let Ok(size_of_control_points) = usize::try_from(number_of_control_points) else {
            return Err(BinaryCurveCollectionParserError::TooManyControlPoints);
        };

        let mut looping = vec![false; size_of_curves].into_boxed_slice();
        let mut first_control_points = vec![0; size_of_curves + 1].into_boxed_slice();
        let mut control_points = vec![0.; size_of_control_points * 3].into_boxed_slice();

        Self::read_curves(
            &mut reader,
            &mut looping,
            &mut first_control_points,
            &mut control_points,
        )?;

        Ok(Self {
            header: BinaryCurveCollectionHeader {
                signature,
                precision,
                curve,
                dimensions,
                up_direction,
                number_of_curves,
                number_of_control_points,
                file_information,
            },
            looping,
            first_control_points,
            control_points,
        })
    }

    /// Parses an [`AsyncRead`] into a [`BinaryCurveCollection`].
    ///
    /// This method has an `sync` counterpart [`BinaryCurveCollection::parse`].
    pub async fn parse_async<T: AsyncRead + Unpin>(
        mut reader: T,
    ) -> Result<Self, BinaryCurveCollectionParserError> {
        debug!("Parsing BinaryCurveCollection");
        let mut signature = [0; 3];
        reader.read_exact(&mut signature).await?;
        if signature != [b'B', b'C', b'C'] {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut precision = 0;
        reader
            .read_exact(std::slice::from_mut(&mut precision))
            .await?;
        if precision != 0x44 {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut curve = [0; 2];
        reader.read_exact(&mut curve).await?;
        if curve != [b'C', b'0'] {
            return Err(BinaryCurveCollectionParserError::InvalidSignature);
        }

        let mut dimensions = 0;
        reader
            .read_exact(std::slice::from_mut(&mut dimensions))
            .await?;

        let mut up_direction = 0;
        reader
            .read_exact(std::slice::from_mut(&mut up_direction))
            .await?;
        if !(1..=2).contains(&up_direction) {
            return Err(BinaryCurveCollectionParserError::InvalidUpDirection);
        }

        let mut number_of_curves = [0; 8];
        reader.read_exact(&mut number_of_curves).await?;
        let number_of_curves = u64::from_le_bytes(number_of_curves);

        let mut number_of_control_points = [0; 8];
        reader.read_exact(&mut number_of_control_points).await?;
        let number_of_control_points = u64::from_le_bytes(number_of_control_points);

        let mut file_information = [0; 40];
        reader.read_exact(&mut file_information).await?;

        let Ok(size_of_curves) = usize::try_from(number_of_curves) else {
            return Err(BinaryCurveCollectionParserError::TooManyCurves);
        };
        let Ok(size_of_control_points) = usize::try_from(number_of_control_points) else {
            return Err(BinaryCurveCollectionParserError::TooManyControlPoints);
        };

        let mut looping = vec![false; size_of_curves].into_boxed_slice();
        let mut first_control_points = vec![0; size_of_curves + 1].into_boxed_slice();
        let mut control_points = vec![0.; size_of_control_points * 3].into_boxed_slice();

        Self::read_curves_async(
            &mut reader,
            &mut looping,
            &mut first_control_points,
            &mut control_points,
        )
        .await?;

        Ok(Self {
            header: BinaryCurveCollectionHeader {
                signature,
                precision,
                curve,
                dimensions,
                up_direction,
                number_of_curves,
                number_of_control_points,
                file_information,
            },
            looping,
            first_control_points,
            control_points,
        })
    }

    /// Read the curves and control points of those curves
    fn read_curves<T: Read>(
        reader: &mut T,
        looping: &mut [bool],
        first_control_points: &mut [usize],
        mut control_points: &mut [f32],
    ) -> Result<(), BinaryCurveCollectionParserError> {
        debug!("Reading curves");
        let mut previous_control_point_start = 0;
        for (looping, first_control_point) in
            looping.iter_mut().zip(first_control_points.iter_mut())
        {
            trace!(
                "Reading a curve. Remaining control points to read: {}.",
                control_points.len()
            );
            let mut curve_control_points = [0; 4];
            reader.read_exact(&mut curve_control_points)?;
            let curve_control_points = i32::from_le_bytes(curve_control_points);

            *looping = curve_control_points < 0;
            *first_control_point = previous_control_point_start;

            let Ok(size) = usize::try_from(curve_control_points.abs()) else {
                return Err(BinaryCurveCollectionParserError::TooManyControlPoints);
            };
            previous_control_point_start += size;

            reader.read_exact(unsafe {
                std::slice::from_raw_parts_mut(
                    control_points[..(size * 3)].as_mut_ptr() as *mut u8,
                    size * 4 * 3,
                )
            })?;
            control_points = &mut control_points[(size * 3)..];
        }
        first_control_points[first_control_points.len() - 1] = previous_control_point_start;

        debug_assert!(control_points.is_empty());

        Ok(())
    }

    /// Read the curves and control points of those curves in `async` context
    async fn read_curves_async<T: AsyncRead + Unpin>(
        reader: &mut T,
        looping: &mut [bool],
        first_control_points: &mut [usize],
        mut control_points: &mut [f32],
    ) -> Result<(), BinaryCurveCollectionParserError> {
        debug!("Reading curves");
        let mut previous_control_point_start = 0;
        for (looping, first_control_point) in
            looping.iter_mut().zip(first_control_points.iter_mut())
        {
            trace!(
                "Reading a curve. Remaining control points to read: {}.",
                control_points.len()
            );
            let mut curve_control_points = [0; 4];
            reader.read_exact(&mut curve_control_points).await?;
            let curve_control_points = i32::from_le_bytes(curve_control_points);

            *looping = curve_control_points < 0;
            *first_control_point = previous_control_point_start;

            let Ok(size) = usize::try_from(curve_control_points.abs()) else {
                return Err(BinaryCurveCollectionParserError::TooManyControlPoints);
            };
            previous_control_point_start += size;

            reader
                .read_exact(unsafe {
                    std::slice::from_raw_parts_mut(
                        control_points[..(size * 3)].as_mut_ptr() as *mut u8,
                        size * 4 * 3,
                    )
                })
                .await?;
            control_points = &mut control_points[(size * 3)..];
        }
        first_control_points[first_control_points.len() - 1] = previous_control_point_start;

        debug_assert!(control_points.is_empty());

        Ok(())
    }
}
