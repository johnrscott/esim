use std::cmp;
use std::fmt;

use csuperlu::sparse_matrix::SparseMat;

use crate::{
    component::Component,
    sparse::{concat_horizontal, concat_vertical},
};

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
pub struct MnaMatrix {
    /// The number of rows in the top matrices
    num_voltage_nodes: usize,
    /// The number of rows in the bottom matrices
    num_current_edges: usize,
    top_left: SparseMat<f64>,
    top_right: SparseMat<f64>,
    bottom_left: SparseMat<f64>,
    bottom_right: SparseMat<f64>,
}

impl MnaMatrix {
    pub fn new() -> Self {
        Self {
            num_voltage_nodes: 0,
            num_current_edges: 0,
            top_left: SparseMat::empty(),
            top_right: SparseMat::empty(),
            bottom_left: SparseMat::empty(),
            bottom_right: SparseMat::empty(),
        }
    }

    pub fn num_voltage_nodes(&self) -> usize {
        self.num_voltage_nodes
    }

    pub fn num_current_edges(&self) -> usize {
        self.num_current_edges
    }

    pub fn get_matrix(mut self) -> SparseMat<f64> {
        self.top_left
            .resize(self.num_voltage_nodes, self.num_voltage_nodes);
        self.bottom_right
            .resize(self.num_current_edges, self.num_current_edges);
        self.top_right
            .resize(self.num_voltage_nodes, self.num_current_edges);
        self.bottom_left
            .resize(self.num_current_edges, self.num_voltage_nodes);

        let top = concat_horizontal(self.top_left, &self.top_right);
        let bottom = concat_horizontal(self.bottom_left, &self.bottom_right);
        concat_vertical(top, &bottom)
    }

    /// Increase the number of voltage nodes if n is not already included. Note
    /// that this function uses the netlist value of n (i.e. the matrix index is
    /// n-1).
    fn update_num_voltage_nodes(&mut self, n: usize) {
        self.num_voltage_nodes = cmp::max(self.num_voltage_nodes, n);
    }

    /// Increase the number of current edges if e is not already included. Note that
    /// e is the actual index into the matrix, so the number of rows will be resized
    /// to e+1
    fn update_num_current_edges(&mut self, e: usize) {
        self.num_current_edges = cmp::max(self.num_current_edges, e + 1);
    }

    /// Add a block of symmetric values to the top-left matrix.
    ///
    /// The two indices specified defines a group of four matrix entries $(n_1-1, n_1-1) =
    /// (n_2-1,n_2-1) = x_1$, and $(n_1-1,n_2-1) = (n_2-1,n_1-1) = x_2$ (i.e. a symmetric block).
    /// Indices $n1$ and $n2$ are non-zero, and must be different. If either
    /// $n_1 = 0$ or $n_2 = 0$, then any elements where the matrix index would
    /// be negative are not written.
    ///
    /// This matrix block is added to the current matrix in the top left of the MNA matrix.
    pub fn add_symmetric_group1(&mut self, n1: usize, n2: usize, x1: f64, x2: f64) {
        if n1 == n2 {
            panic!("Cannot set symmetric group 1 where n1 == n2");
        }
        self.update_num_voltage_nodes(n1);
        self.update_num_voltage_nodes(n2);
        if n1 == 0 {
            plus_equals(&mut self.top_left, n2 - 1, n2 - 1, x1);
        } else if n2 == 0 {
            plus_equals(&mut self.top_left, n1 - 1, n1 - 1, x1);
        } else {
            plus_equals(&mut self.top_left, n1 - 1, n1 - 1, x1);
            plus_equals(&mut self.top_left, n2 - 1, n2 - 1, x1);
            plus_equals(&mut self.top_left, n1 - 1, n2 - 1, x2);
            plus_equals(&mut self.top_left, n2 - 1, n1 - 1, x2);
        }
    }

    /// Add a symmetric component into the off-diagonal blocks and bottom-left matrix
    ///
    /// The function accumulates: $x_1$ to $(n_1-1, e)$ (top-right) and $(e, n_1-1)$
    /// (bottom-left); $x_2$ to $(n_2-1, e)$ (top-right) and $(e, n_2-1)$
    /// (bottom-left); and $y$ to $(e, e)$ (bottom-right).
    ///
    /// In all cases, if all cases, $n_1 != n_2$, and if $n_1 = 0$ or $n_2 = 0$, then
    /// the corresponding matrix entries are not written.
    pub fn add_symmetric_group2(
        &mut self,
        n1: usize,
        n2: usize,
        e: usize,
        x1: f64,
        x2: f64,
        y: f64,
    ) {
        if n1 == n2 {
            panic!("Cannot set symmetric group 2 where n1 == n2");
        }
        self.update_num_voltage_nodes(n1);
        self.update_num_voltage_nodes(n2);
        self.update_num_current_edges(e);
        plus_equals(&mut self.bottom_right, e, e, y);
        if n1 != 0 {
            plus_equals(&mut self.top_right, n1 - 1, e, x1);
            plus_equals(&mut self.bottom_left, e, n1 - 1, x1);
        }
        if n2 != 0 {
            plus_equals(&mut self.top_right, n2 - 1, e, x2);
            plus_equals(&mut self.bottom_left, e, n2 - 1, x2);
        }
    }

    /// Same as symmetric version, but only adds values to the
    /// right-hand portion of the matrix (top and bottom)
    pub fn add_unsymmetric_right_group2(
        &mut self,
        n1: usize,
        n2: usize,
        e: usize,
        x1: f64,
        x2: f64,
        y: f64,
    ) {
        if n1 == n2 {
            panic!("Cannot set unsymmetric group (right) 2 where n1 == n2");
        }
        self.update_num_voltage_nodes(n1);
        self.update_num_voltage_nodes(n2);
        self.update_num_current_edges(e);
        plus_equals(&mut self.bottom_right, e, e, y);
        if n1 != 0 {
            plus_equals(&mut self.top_right, n1 - 1, e, x1);
        }
        if n2 != 0 {
            plus_equals(&mut self.top_right, n2 - 1, e, x2);
        }
    }

    /// Same as symmetric version, but only adds values to the
    /// bottom portion of the matrix (left and right)
    pub fn add_unsymmetric_bottom_group2(
        &mut self,
        n1: usize,
        n2: usize,
        e: usize,
        x1: f64,
        x2: f64,
        y: f64,
    ) {
        if n1 == n2 {
            panic!("Cannot set unsymmetric group (bottom) 2 where n1 == n2");
        }
        self.update_num_voltage_nodes(n1);
        self.update_num_voltage_nodes(n2);
        self.update_num_current_edges(e);
        plus_equals(&mut self.bottom_right, e, e, y);
        if n1 != 0 {
            plus_equals(&mut self.bottom_left, e, n1 - 1, x1);
        }
        if n2 != 0 {
            plus_equals(&mut self.bottom_left, e, n2 - 1, x2);
        }
    }

    /// Add a single value in the group2 (current-current, bottom-right) portion
    /// of the matrix
    pub fn add_group2_value(
        &mut self,
        e1: usize,
        e2: usize,
        y: f64,
    ) {
        self.update_num_current_edges(e1);
        self.update_num_current_edges(e2);
        plus_equals(&mut self.bottom_right, e1, e2, y);
    }
}

/// Modified nodal analysis right-hand side
///
/// The right-hand side for modified nodal analysis is
///
/// | -A1 s1 |
/// |        |
/// |   s2   |
///
pub struct MnaRhs {
    top: SparseMat<f64>,
    bottom: SparseMat<f64>,
}

/// Assumes the matrix is square
fn plus_equals(mat: &mut SparseMat<f64>, row: usize, col: usize, val: f64) {
    let old_val = mat.get_unbounded(row, col);
    mat.insert_unbounded(row, col, old_val + val);
}

impl fmt::Display for MnaMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Num voltage nodes = {}, Num current edges = {}",
            self.num_voltage_nodes, self.num_current_edges
        )?;
        writeln!(f, "Top left:")?;
        writeln!(f, "{}", self.top_left)?;
        writeln!(f, "Top right:")?;
        writeln!(f, "{}", self.top_right)?;
        writeln!(f, "Bottom left:")?;
        writeln!(f, "{}", self.bottom_left)?;
        writeln!(f, "Bottom right:")?;
        writeln!(f, "{}", self.bottom_right)
    }
}

impl MnaRhs {
    fn new() -> Self {
        Self {
            top: SparseMat::empty(),
            bottom: SparseMat::empty(),
        }
    }

    pub fn get_vector(self, num_voltage_nodes: usize, num_current_edges: usize) -> Vec<f64> {
        let mut out = vec![0.0; num_voltage_nodes + num_current_edges];
        for ((row, _), value) in self.top.non_zero_vals().iter() {
            out[*row] = *value;
        }
        for ((row, _), value) in self.bottom.non_zero_vals().iter() {
            out[num_voltage_nodes + *row] = *value;
        }
        out
    }

    /// Add a RHS element in the group 1 matrix.
    pub fn add_rhs_group1(&mut self, n: usize, x: f64) {
	if n != 0 {
            self.top.insert_unbounded(n - 1, 1, x);
	}
    }
    
    /// Add a RHS element in the group 2 matrix
    pub fn add_rhs_group2(&mut self, e: usize, x: f64) {
        self.bottom.insert_unbounded(e, 1, x);
    }
}

pub struct Mna {
    matrix: MnaMatrix,
    rhs: MnaRhs,
}

impl Mna {
    pub fn new() -> Self {
        Self {
            matrix: MnaMatrix::new(),
            rhs: MnaRhs::new(),
        }
    }

    pub fn num_voltage_nodes(&self) -> usize {
        self.matrix.num_voltage_nodes()
    }

    pub fn num_current_edges(&self) -> usize {
        self.matrix.num_current_edges()
    }

    pub fn add_element_stamp(&mut self, component: &Component) {
        match component {
            Component::Resistor {
                term_1,
                term_2,
                current_index,
                resistance: r,
            } => {
                match current_index {
                    Some(edge) => self
                        .matrix
                        .add_symmetric_group2(*term_1, *term_2, *edge, 1.0, -1.0, -*r),
                    None => self
                        .matrix
                        .add_symmetric_group1(*term_1, *term_2, 1.0 / r, -1.0 / r),
                }
            },
            Component::IndependentVoltageSource {
                term_pos,
                term_neg,
                current_index,
                voltage: v,
            } => {
                self.matrix.add_symmetric_group2(
                    *term_pos,
                    *term_neg,
                    *current_index,
                    1.0,
                    -1.0,
                    0.0,
                );
                self.rhs.add_rhs_group2(*current_index, *v);
            },
            Component::VoltageControlledVoltageSource {
                term_pos,
                term_neg,
		ctrl_pos,
		ctrl_neg,
                current_index,
                voltage_scale: k,
            } => {
                self.matrix.add_symmetric_group2(
                    *term_pos,
                    *term_neg,
                    *current_index,
                    1.0,
                    -1.0,
                    0.0,
                );
		self.matrix.add_unsymmetric_bottom_group2(
		    *ctrl_pos,
		    *ctrl_neg,
		    *current_index,
		    -*k,
		    *k,
		    0.0,
		);
            },
            Component::CurrentControlledVoltageSource {
                term_pos,
                term_neg,
		ctrl_edge,
                current_index,
                voltage_scale: k,
            } => {
                self.matrix.add_symmetric_group2(
                    *term_pos,
                    *term_neg,
                    *current_index,
                    1.0,
                    -1.0,
                    0.0,
                );
		self.matrix.add_group2_value(*current_index, *ctrl_edge, -*k);
            },
	    Component::IndependentCurrentSource {
                term_pos,
                term_neg,
                current_index,
                current: i
            } => {
                match current_index {
                    Some(edge) => {
			self.matrix.add_unsymmetric_right_group2(
			    *term_pos, *term_neg, *edge,
			    1.0, -1.0, 1.0);
			self.rhs.add_rhs_group2(*edge, *i);
		    },
                    None => {
			self.rhs.add_rhs_group1(*term_pos, -*i);
			self.rhs.add_rhs_group1(*term_neg, *i);
		    }
                }
            },
            _ => todo!("Not currently implemented"),
        }
    }

    /// Return (matrix, rhs)
    pub fn get_system(self) -> (SparseMat<f64>, Vec<f64>) {
        let num_voltage_nodes = self.matrix.num_voltage_nodes();
        let num_current_edges = self.matrix.num_current_edges();
        let matrix = self.matrix.get_matrix();
        let rhs = self.rhs.get_vector(num_voltage_nodes, num_current_edges);
        (matrix, rhs)
    }
}

impl fmt::Display for Mna {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "MNA matrix:")?;
        writeln!(f, "{}", self.matrix)
    }
}
