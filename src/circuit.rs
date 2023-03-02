use csuperlu::c::value_type::ValueType;
use csuperlu::sparse_matrix::SparseMatrix;

use crate::sparse::{concat_horizontal, concat_vertical,
		    transpose};
use crate::circuit::instance::Instance;

/// Matrix for modified nodal analysis
///
/// Stores the modified nodal analysis matrix
/// for a resistive network with no controlled,
/// sources, where group 2 contains no current
/// sources.
///
///  | A1 Y11 A1^T     A2  |
///  |                     |
///  |   - A2         Z22  |
/// 
///
struct MnaMatrix<P: ValueType<P>> {
    a1_y11_a1t: SparseMatrix<P>,
    a2: SparseMatrix<P>,
    z22: SparseMatrix<P>,
}

/// Modified nodal analysis right-hand side
///
/// The right-hand side for modified nodal analysis is
///
/// | -A1 s1 |
/// |        |
/// |   s2   |
///
struct MnaRhs<P: ValueType<P>> {
    minus_a1_s1: Vec<P>,
    s2: Vec<P>,
}

impl<P: ValueType<P>> MnaMatrix<P> {
    pub fn new() -> Self {
	Self {
	    // Insert some placeholder size here
	    a1_y11_a1t: SparseMatrix::new(0, 0),
	    a2: SparseMatrix::new(0, 0),
	    z22: SparseMatrix::new(0, 0),
	}
    }
    pub fn get_matrix(self) -> SparseMatrix<P> {
	let top = concat_horizontal(self.A1Y11A1t, &self.A2);
	let bottom = concat_horizontal(-transpose(self.A2), &self.Z22);
	concat_vertical(top, &bottom)
    }
    pub fn insert_group1(&self, row: usize, col: usize, value: f64) {
	self.a1_y11_a1t.set_value(row, col, value);
    }
    pub fn insert_group2(&self, row: usize, col: usize, value: f64) {
	self.a1_y11_a1t.set_value(row, col, value);
    }
}

impl<P: ValueType<P>> MnaRhs<P> {
    fn new() -> Self {
	Self {
	    minus_a1_s1: Vec::new(),
	    s2: Vec::new(),
	}
    }
    pub fn get_vector(mut self) -> Vec<P> {
	self.top.append(self.bottom);
	self.top
    }
}

pub struct Circuit<P: ValueType<P>> {
    instances: Vec<Instance>,
    mna_matrix: MnaMatrix<P>,
    mna_rhs: MnaRhs<P>,
}

impl<P: ValueType<P>> Circuit<P> {
   pub fn new() -> Self {
	Self {
	    instances: Vec::new(),
	    mna_matrix: MnaMatrix::new(),
	    mna_rhs: MnaRhs::new(),
	}
    }
}