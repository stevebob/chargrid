use super::{Coord, Size};
use crate::context::*;
use crate::view_cell::*;

pub fn set_cell_relative_to_draw<F: ?Sized + Frame, C: ColModify>(
    frame: &mut F,
    relative_coord: Coord,
    relative_depth: i32,
    relative_cell: ViewCell,
    context: ViewContext<C>,
) {
    let adjusted_relative_coord = relative_coord + context.inner_offset;
    if adjusted_relative_coord.is_valid(context.size) {
        let absolute_coord = adjusted_relative_coord + context.outer_offset;
        let absolute_depth = relative_depth + context.depth;
        let absolute_cell = ViewCell {
            style: Style {
                foreground: relative_cell
                    .style
                    .foreground
                    .map(|rgb24| context.col_modify.modify(rgb24)),
                background: relative_cell
                    .style
                    .background
                    .map(|rgb24| context.col_modify.modify(rgb24)),
                ..relative_cell.style
            },
            ..relative_cell
        };
        frame.set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

pub fn set_cell_relative_to_measure_size<F: ?Sized + Frame, C: ColModify>(
    frame: &mut F,
    relative_coord: Coord,
    context: ViewContext<C>,
) {
    let adjusted_relative_coord = relative_coord + context.inner_offset;
    let absolute_coord = adjusted_relative_coord + context.outer_offset;
    const DEFAULT_CELL: ViewCell = ViewCell::new();
    frame.set_cell_absolute(absolute_coord, 0, DEFAULT_CELL);
}

/// A frame of animation
pub trait Frame {
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i32,
        relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        set_cell_relative_to_draw(self, relative_coord, relative_depth, relative_cell, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i32, absolute_cell: ViewCell);
}

#[derive(Debug)]
pub struct MeasureBounds {
    max_coord: Coord,
}

impl MeasureBounds {
    pub fn new() -> Self {
        Self {
            max_coord: Coord::new(0, 0),
        }
    }
    pub fn bounds(&self, outer_offset: Coord) -> Size {
        (self.max_coord - outer_offset).to_size().unwrap_or(Size::new_u16(0, 0)) + Size::new_u16(1, 1)
    }
}

impl Frame for MeasureBounds {
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        _relative_depth: i32,
        _relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        set_cell_relative_to_measure_size(self, relative_coord, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, _absolute_depth: i32, _absolute_cell: ViewCell) {
        self.max_coord.x = self.max_coord.x.max(absolute_coord.x);
        self.max_coord.y = self.max_coord.y.max(absolute_coord.y);
    }
}

pub struct MeasureBoundsAndDraw<'a, F> {
    frame: &'a mut F,
    measure_bounds: MeasureBounds,
}

impl<'a, F> MeasureBoundsAndDraw<'a, F>
where
    F: Frame,
{
    pub fn new(frame: &'a mut F) -> Self {
        Self {
            frame,
            measure_bounds: MeasureBounds::new(),
        }
    }
    pub fn bounds(&self, outer_offset: Coord) -> Size {
        self.measure_bounds.bounds(outer_offset)
    }
}

impl<'a, F> Frame for MeasureBoundsAndDraw<'a, F>
where
    F: Frame,
{
    fn set_cell_relative<C: ColModify>(
        &mut self,
        relative_coord: Coord,
        relative_depth: i32,
        relative_cell: ViewCell,
        context: ViewContext<C>,
    ) {
        self.frame
            .set_cell_relative(relative_coord, relative_depth, relative_cell, context);
        self.measure_bounds
            .set_cell_relative(relative_coord, relative_depth, relative_cell, context);
    }
    fn set_cell_absolute(&mut self, absolute_coord: Coord, absolute_depth: i32, absolute_cell: ViewCell) {
        self.frame
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
        self.measure_bounds
            .set_cell_absolute(absolute_coord, absolute_depth, absolute_cell);
    }
}

pub trait View<T> {
    /// Update the cells in `frame` to describe how a type should be rendered.
    /// This mutably borrows `self` to allow the view to contain buffers/caches which
    /// are updated during rendering.
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F);

    fn visible_bounds<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        let mut measure_bounds = MeasureBounds::new();
        self.view(data, context, &mut measure_bounds);
        measure_bounds.bounds(context.outer_offset)
    }

    /// Render an element and return the size that the element, regardless of the
    /// size of the visible component of the element. This allows decorators to know
    /// the size of the output of a view they are decorating.
    /// By default this calls `view` keeping track of the maximum x and y
    /// components of the relative coords of cells which are set in `frame`.
    fn view_reporting_intended_size<F: Frame, C: ColModify>(
        &mut self,
        data: T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> Size {
        let mut measure_bounds_and_draw = MeasureBoundsAndDraw::new(frame);
        self.view(data, context, &mut measure_bounds_and_draw);
        measure_bounds_and_draw.bounds(context.outer_offset)
    }
}

impl<'a, T, V: View<T>> View<T> for &'a mut V {
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        (*self).view(data, context, frame)
    }
}
