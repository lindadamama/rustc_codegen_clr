use fxhash::FxHashSet;

use crate::{bimap::IntoBiMapIndex, IString};

use super::{
    bimap::Interned,
    cilnode::{PtrCastRes, UnOp},
    method::LocalDef,
    Assembly, BinOp, CILNode, CILRoot, ClassRef, FieldDesc, FnSig, Int, Type,
};
#[derive(Debug)]
/// Signals that a piece of CIL is not valid.
pub enum TypeCheckError {
    /// CIL contains a binop with incorrect arguments
    WrongBinopArgs {
        /// The type of the left argument of this op
        lhs: Type,
        /// The type of the right argument of this op
        rhs: Type,
        /// The type of this op
        op: BinOp,
    },
    /// A reference-to-pointer cast is not a reference
    RefToPtrArgNotRef {
        /// The non-reference type encountered.
        arg: Type,
    },
    /// Incorrect pointer cast
    InvalidPtrCast {
        /// The result of this cast
        expected: PtrCastRes,
        /// The source type
        got: Type,
    },
    /// A non-pointer type was passed to an instruction expecting a pointer type
    TypeNotPtr {
        /// The incorrect type
        tpe: Type,
    },
    /// A load instruction was passed an incorrect type.
    DerfWrongPtr {
        /// Expected type
        expected: Type,
        /// Received type
        got: Type,
    },
    /// A call instruction was passed a wrong amount of args.
    CallArgcWrong {
        /// The signature-specified amount of args
        expected: usize,
        /// The received amount of args.
        got: usize,
        /// The name of this method
        mname: IString,
    },
    /// A call instruction was passed a wrong argument type.
    CallArgTypeWrong {
        /// The received type
        got: String,
        /// The expected type
        expected: String,
        /// The index of this argument
        idx: usize,
        /// The called method
        mname: IString,
    },
    IntCastInvalidInput {
        got: Type,
        target: Int,
    },
    /// Attempted to access the field of a type without fields.
    FieldAccessInvalidType {
        tpe: Type,
        field: crate::FieldDesc,
    },
    FieldOwnerMismatch {
        owner: Interned<ClassRef>,
        expected_owner: Interned<ClassRef>,
        field: crate::FieldDesc,
    },
    ExpectedClassGotValuetype {
        cref: ClassRef,
    },
    TypeNotClass {
        object: Type,
    },
    FloatCastInvalidInput {
        got: Type,
        target: super::Float,
    },
    WrongUnOpArgs {
        tpe: Type,
        op: UnOp,
    },
    /// Incorrect amount of args to an indirect call
    IndirectCallArgcWrong {
        expected: usize,
        got: usize,
    },
    /// An incorrect argument to an indirect call
    IndirectCallArgTypeWrong {
        got: Type,
        expected: Type,
        idx: usize,
    },
    /// Attempted to get the length of a non-array type
    LdLenArgNotArray {
        /// The non-array type
        got: Type,
    },
    /// Attempted to get the length of a managed array with a more than one dimension.
    LdLenArrNot1D {
        /// Array with dimension mismatch
        got: Type,
    },
    /// Invalid index into a managed array
    ArrIndexInvalidType {
        /// Received index type
        index_tpe: Type,
    },
    /// An indirect call with a non-fn-pointer type
    IndirectCallInvalidFnPtrType {
        /// non-fn-pointer-type
        fn_ptr: Type,
    },
    /// An indirect call with a mismatching signature
    IndirectCallInvalidFnPtrSig {
        /// Expected signature
        expected: super::FnSig,
        /// Signature of the pointer
        got: super::FnSig,
    },
    /// Atempt to calculate the size of void.
    SizeOfVoid,
    /// Asigned a wrong type to a local variable.
    LocalAssigementWrong {
        /// Index of the local.
        loc: u32,
        /// Received type.
        got: String,
        /// Expected type
        expected: String,
    },
    /// A comparison of non-prmitive types.
    ValueTypeCompare {
        /// Lhs side of the compare
        lhs: Type,
        /// Rhs side of the compare
        rhs: Type,
    },
    /// A write instruction was passed an address of incorrect type.
    WriteWrongAddr {
        /// Expected addr type
        addr: String,
        /// Received type
        tpe: String,
    },
    /// A write instruction was passed a value of incorrect type.
    WriteWrongValue {
        /// The expected type
        tpe: Type,
        /// The received type.
        value: Type,
    },
    /// Incorrect argument to a branch instruction
    ConditionNotBool {
        /// The wrong, not-bool type.
        cond: Type,
    },
    /// A comparsion instruction was used on a pair of types that can't be compared.
    CantCompareTypes {
        /// Lhs type
        lhs: Type,
        /// Rhs type
        rhs: Type,
    },
    /// A field assigement instruction was passed an icorrect type.
    FieldAssignWrongType {
        /// The expected type
        field_tpe: Type,
        /// The reference to the field.
        fld: Interned<FieldDesc>,
        /// The received type.
        val: Type,
    },
    /// An instruction attempted to access a field that does not exist.
    FieldNotPresent {
        /// The type of the field.
        tpe: Type,
        /// The name of the field.
        name: super::Interned<IString>,
        /// The owner of this field.
        owner: super::Interned<ClassRef>,
    },
    /// An operation was performed on a void pointer.
    VoidPointerOp {
        /// The kind of operation that was done.
        op: BinOp,
    },
    ManagedPtrCast {
        src: String,
        dst: String,
    },
}
/// Converts a typecheck error to a graph representing the issue with the typecheck process.
pub fn typecheck_err_to_string(
    root_idx: super::Interned<CILRoot>,
    asm: &mut Assembly,
    sig: Interned<FnSig>,
    locals: &[LocalDef],
) -> String {
    let root = asm[root_idx].clone();
    let mut set = FxHashSet::default();
    let nodes = root
        .nodes()
        .iter()
        .map(|node| display_node(**node, asm, sig, locals, &mut set))
        .collect::<String>();
    let root_connections: String = root.nodes().iter().fold(String::new(), |mut output, node| {
        use std::fmt::Write;
        writeln!(output, "n{node} ", node = node.as_bimap_index()).unwrap();
        output
    });
    let root_string = root.display(asm, sig, locals);
    match root.typecheck(sig, locals, asm){
        Ok(_)=> format!("digraph G{{edge [dir=\"back\"];\n{nodes} r{root_idx}  [label = \"{root_string}\" color = \"green\"] r{root_idx} ->{root_connections}}}",root_idx = root_idx.as_bimap_index()),
        Err(err)=> format!("digraph G{{edge [dir=\"back\"];\\n{nodes} r{root_idx}  [label = \"{root_string}\n{err:?}\" color = \"red\"] r{root_idx} ->{root_connections}}}",root_idx = root_idx.as_bimap_index()),
   }
}
/// Display an error during typechecking root `root_idx`.
pub fn display_typecheck_err(
    root_idx: super::Interned<CILRoot>,
    asm: &mut Assembly,
    sig: Interned<FnSig>,
    locals: &[LocalDef],
) {
    eprintln!("{}", typecheck_err_to_string(root_idx, asm, sig, locals))
}
#[doc(hidden)]
pub fn display_node(
    nodeidx: Interned<CILNode>,
    asm: &mut Assembly,
    sig: Interned<FnSig>,
    locals: &[LocalDef],
    set: &mut FxHashSet<Interned<CILNode>>,
) -> String {
    let node = asm.get_node(nodeidx).clone();
    set.insert(nodeidx);
    let tpe = node.typecheck(sig, locals, asm);
    let node_def = match tpe {
        Ok(tpe) => format!(
            "n{nodeidx} [label = {node:?} color = \"green\"]",
            nodeidx = nodeidx.as_bimap_index(),
            node = format!("{node:?}\n{}", tpe.mangle(asm))
        ),
        Err(err) => format!(
            "n{nodeidx} [label = {node:?} color = \"red\"]",
            nodeidx = nodeidx.as_bimap_index(),
            node = format!("{node:?}\n{err:?}")
        ),
    };
    let node_children = node.child_nodes();
    let node_children_str: String = node_children
        .iter()
        .fold(String::new(), |mut output, node| {
            use std::fmt::Write;
            let _ = write!(output, " n{nodeidx} ", nodeidx = node.as_bimap_index(),);
            output
        });
    if node_children.is_empty() {
        format!("{node_def}\n")
    } else {
        let mut res = format!(
            "{node_def}\n n{nodeidx}  -> {{{node_children_str}}}\n",
            nodeidx = nodeidx.as_bimap_index(),
        );
        for nodeidx in node.child_nodes() {
            res.push_str(&display_node(nodeidx, asm, sig, locals, set));
        }
        res
    }
}
impl BinOp {
    fn typecheck(&self, lhs: Type, rhs: Type, asm: &Assembly) -> Result<Type, TypeCheckError> {
        match self {
            BinOp::Add | BinOp::Sub => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs => Ok(Type::Int(lhs)),
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Float(lhs)),
                (Type::Ptr(lhs), Type::Ptr(rhs)) if rhs == lhs => Ok(Type::Ptr(lhs)),
                (Type::FnPtr(lhs), Type::FnPtr(rhs)) if rhs == lhs => Ok(Type::FnPtr(lhs)),
                (Type::Ptr(_inner), Type::Int(Int::ISize | Int::USize)) => {
                    // Since pointer ops operate in bytes, this is not an issue ATM.
                    /*if asm[inner] != Type::Void {
                        Ok(lhs)
                    } else {
                        Err(TypeCheckError::VoidPointerOp { op: self.clone() })
                    }*/
                    Ok(lhs)
                }
                (Type::FnPtr(_), Type::Int(Int::ISize | Int::USize)) => Ok(lhs),
                (Type::Int(Int::ISize | Int::USize), Type::Ptr(_) | Type::FnPtr(_)) => Ok(rhs),
                // TODO: investigate the cause of this issue. Changing a reference is not valid.
                (Type::Ref(_), Type::Int(Int::ISize | Int::USize)) => Ok(lhs),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Eq => {
                if lhs == rhs || lhs.is_assignable_to(rhs, asm) {
                    if let Type::ClassRef(cref) = lhs {
                        if asm[cref].is_valuetype() {
                            Err(TypeCheckError::ValueTypeCompare { lhs, rhs })
                        } else {
                            Ok(Type::Bool)
                        }
                    } else {
                        Ok(Type::Bool)
                    }
                } else {
                    Err(TypeCheckError::WrongBinopArgs {
                        lhs,
                        rhs,
                        op: *self,
                    })
                }
            }

            BinOp::Mul => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs => Ok(Type::Int(lhs)),
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Float(lhs)),
                (Type::Int(Int::ISize | Int::USize), Type::Ptr(_) | Type::FnPtr(_)) => Ok(rhs),
                // Relaxes the rules to prevent some wierd issue with sizeof
                (Type::Int(Int::ISize), Type::Int(Int::I32)) => Ok(Type::Int(Int::ISize)),
                (Type::Int(Int::USize), Type::Int(Int::I32)) => Ok(Type::Int(Int::USize)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm) {
                        Ok(rhs)
                    } else if rhs.is_assignable_to(lhs, asm) {
                        Ok(lhs)
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::LtUn | BinOp::GtUn => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::Ptr(lhs), Type::Ptr(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::FnPtr(lhs), Type::FnPtr(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::Bool, Type::Bool) => Ok(Type::Bool),
                _ => {
                    if lhs == rhs || lhs.is_assignable_to(rhs, asm) {
                        Ok(Type::Bool)
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Lt | BinOp::Gt => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Bool),
                (Type::Bool, Type::Bool) => Ok(Type::Bool),
                _ => {
                    if lhs == rhs || lhs.is_assignable_to(rhs, asm) {
                        Ok(Type::Bool)
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Or | BinOp::XOr | BinOp::And => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs => Ok(Type::Int(lhs)),
                (Type::Bool, Type::Bool) => Ok(Type::Bool),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Rem => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs && rhs.is_signed() => {
                    Ok(Type::Int(lhs))
                }
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Bool),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::RemUn => match (lhs, rhs) {
                (Type::Int(lhs), Type::Int(rhs)) if rhs == lhs && !rhs.is_signed() => {
                    Ok(Type::Int(lhs))
                }
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Bool),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Shl => match (lhs, rhs) {
                (
                    Type::Int(
                        lhs @ (Int::I128
                        | Int::U128
                        | Int::I64
                        | Int::U64
                        | Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8),
                    ),
                    Type::Int(
                        Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8,
                    ),
                ) => Ok(Type::Int(lhs)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Shr => match (lhs, rhs) {
                (
                    Type::Int(
                        lhs @ (Int::I128
                        | Int::U128
                        | Int::I64
                        | Int::U64
                        | Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8),
                    ),
                    Type::Int(
                        Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8,
                    ),
                ) if lhs.is_signed() => Ok(Type::Int(lhs)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::ShrUn => match (lhs, rhs) {
                (
                    Type::Int(
                        lhs @ (Int::I128
                        | Int::U128
                        | Int::I64
                        | Int::U64
                        | Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8),
                    ),
                    Type::Int(
                        Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8,
                    ),
                ) if !lhs.is_signed() => Ok(Type::Int(lhs)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::DivUn => match (lhs, rhs) {
                (
                    Type::Int(lhs @ (Int::U64 | Int::USize | Int::U32 | Int::U16 | Int::U8)),
                    Type::Int(rhs @ (Int::U64 | Int::USize | Int::U32 | Int::U16 | Int::U8)),
                ) if lhs == rhs => Ok(Type::Int(lhs)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
            BinOp::Div => match (lhs, rhs) {
                (
                    Type::Int(
                        lhs @ (Int::U64
                        | Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8),
                    ),
                    Type::Int(
                        rhs @ (Int::U64
                        | Int::USize
                        | Int::ISize
                        | Int::I32
                        | Int::U32
                        | Int::I16
                        | Int::U16
                        | Int::U8
                        | Int::I8),
                    ),
                ) if lhs.is_signed() && lhs == rhs => Ok(Type::Int(lhs)),
                (Type::Float(lhs), Type::Float(rhs)) if rhs == lhs => Ok(Type::Float(lhs)),
                _ => {
                    if lhs.is_assignable_to(rhs, asm)
                        && (lhs.as_int().is_some() || rhs.as_int().is_some())
                    {
                        Ok(Type::Int(lhs.as_int().or(rhs.as_int()).unwrap()))
                    } else {
                        Err(TypeCheckError::WrongBinopArgs {
                            lhs,
                            rhs,
                            op: *self,
                        })
                    }
                }
            },
        }
    }
}
impl CILNode {
    #[allow(unused_variables)]
    /// Typechecks this node, and returns its type if its valid.
    /// # Errors
    /// Returns an error if this node can't pass type checks.
    pub fn typecheck(
        &self,
        sig: Interned<FnSig>,
        locals: &[LocalDef],
        asm: &mut Assembly,
    ) -> Result<Type, TypeCheckError> {
        match self {
            CILNode::Const(cst) => Ok(cst.as_ref().get_type()),
            CILNode::BinOp(lhs, rhs, op) => {
                let lhs = asm.get_node(*lhs).clone();
                let rhs = asm.get_node(*rhs).clone();
                let lhs = lhs.typecheck(sig, locals, asm)?;
                let rhs = rhs.typecheck(sig, locals, asm)?;
                op.typecheck(lhs, rhs, asm)
            }
            CILNode::UnOp(arg, op) => {
                let arg = asm.get_node(*arg).clone();
                let arg_type = arg.typecheck(sig, locals, asm)?;
                match (arg_type, op) {
                    (Type::Int(_) | Type::Float(_) | Type::Ptr(_), UnOp::Not) => Ok(arg_type),
                    (Type::Int(int), UnOp::Neg) if int.is_signed() => Ok(arg_type),
                    (Type::Float(_) | Type::Ptr(_), UnOp::Neg) => Ok(arg_type),
                    _ => Err(TypeCheckError::WrongUnOpArgs {
                        tpe: arg_type,
                        op: op.clone(),
                    }),
                }
            }
            CILNode::LdLoc(loc) => Ok(asm[locals[*loc as usize].1]),
            CILNode::LdLocA(loc) => Ok(asm.nref(asm[locals[*loc as usize].1])),
            CILNode::LdArg(arg) => Ok(asm[sig].inputs()[*arg as usize]),
            CILNode::LdArgA(arg) => Ok(asm.nref(asm[sig].inputs()[*arg as usize])),
            CILNode::Call(call_info) => {
                let (mref, args, _is_pure) = call_info.as_ref();
                let mref = asm[*mref].clone();
                let inputs: Box<[_]> = mref.stack_inputs(asm).into();
                if args.len() != inputs.len() {
                    return Err(TypeCheckError::CallArgcWrong {
                        expected: inputs.len(),
                        got: args.len(),
                        mname: asm[mref.name()].into(),
                    });
                }
                for (idx, (arg, input_type)) in args.iter().zip(inputs.iter()).enumerate() {
                    let arg = asm.get_node(*arg).clone();
                    let arg_type = arg.typecheck(sig, locals, asm)?;
                    if !arg_type.is_assignable_to(*input_type, asm)
                        && !arg_type
                            .try_deref(asm)
                            .is_some_and(|t| Some(t) == input_type.try_deref(asm))
                    {
                        return Err(TypeCheckError::CallArgTypeWrong {
                            got: arg_type.mangle(asm),
                            expected: input_type.mangle(asm),
                            idx,
                            mname: asm[mref.name()].into(),
                        });
                    }
                }
                Ok(mref.output(asm))
            }
            CILNode::CallI(info) => {
                let (fn_ptr, called_sig, args) = info.as_ref();
                let fn_ptr = asm.get_node(*fn_ptr).clone();
                let fn_ptr = fn_ptr.typecheck(sig, locals, asm)?;
                let called_sig = asm[*called_sig].clone();
                if args.len() != called_sig.inputs().len() {
                    return Err(TypeCheckError::IndirectCallArgcWrong {
                        expected: called_sig.inputs().len(),
                        got: args.len(),
                    });
                }

                for (idx, (arg, input_type)) in
                    args.iter().zip(called_sig.inputs().iter()).enumerate()
                {
                    let arg = asm.get_node(*arg).clone();
                    let arg_type = arg.typecheck(sig, locals, asm)?;
                    if !arg_type.is_assignable_to(*input_type, asm) {
                        return Err(TypeCheckError::IndirectCallArgTypeWrong {
                            got: arg_type,
                            expected: *input_type,
                            idx,
                        });
                    }
                }
                let Type::FnPtr(ptr_sig) = fn_ptr else {
                    return Err(TypeCheckError::IndirectCallInvalidFnPtrType { fn_ptr });
                };
                let ptr_sig = &asm[ptr_sig];
                if *ptr_sig != called_sig {
                    return Err(TypeCheckError::IndirectCallInvalidFnPtrSig {
                        expected: called_sig,
                        got: ptr_sig.clone(),
                    });
                }
                Ok(*called_sig.output())
            }
            CILNode::IntCast {
                input,
                target,
                extend,
            } => {
                let input = asm.get_node(*input).clone();
                let input = input.typecheck(sig, locals, asm)?;
                match input {
                    Type::Float(_) | Type::Int(_) | Type::Ptr(_) | Type::FnPtr(_) | Type::Bool => {
                        Ok(Type::Int(*target))
                    }
                    _ => Err(TypeCheckError::IntCastInvalidInput {
                        got: input,
                        target: *target,
                    }),
                }
            }
            CILNode::FloatCast {
                input,
                target,
                is_signed,
            } => {
                let input = asm.get_node(*input).clone();
                let input = input.typecheck(sig, locals, asm)?;
                match input {
                    Type::Float(_) | Type::Int(_) => Ok(Type::Float(*target)),
                    _ => Err(TypeCheckError::FloatCastInvalidInput {
                        got: input,
                        target: *target,
                    }),
                }
            }
            CILNode::RefToPtr(refn) => {
                let refn = asm.get_node(*refn).clone();
                let tpe = refn.typecheck(sig, locals, asm)?;
                match tpe {
                    Type::Ref(inner) | Type::Ptr(inner) => Ok(asm.nptr(asm[inner])),
                    _ => Err(TypeCheckError::RefToPtrArgNotRef { arg: tpe }),
                }
            }
            CILNode::PtrCast(arg, res) => {
                let arg = asm.get_node(*arg).clone();
                let arg_tpe = arg.typecheck(sig, locals, asm)?;
                match arg_tpe {
                    Type::Ptr(inner) | Type::Ref(inner) => {
                        if asm[inner].is_gcref(asm) {
                            return Err(TypeCheckError::ManagedPtrCast {
                                src: arg_tpe.mangle(asm),
                                dst: res.as_ref().as_type().mangle(asm),
                            });
                        }
                    }

                    Type::Int(Int::USize | Int::ISize) | Type::FnPtr(_) => (),
                    _ => Err(TypeCheckError::InvalidPtrCast {
                        expected: res.as_ref().clone(),
                        got: arg_tpe,
                    })?,
                };
                if res.as_ref().as_type().is_gcref(asm) {
                    return Err(TypeCheckError::ManagedPtrCast {
                        src: arg_tpe.mangle(asm),
                        dst: res.as_ref().as_type().mangle(asm),
                    });
                }
                Ok(res.as_ref().as_type())
            }
            CILNode::LdFieldAddress { addr, field } => {
                let field = *asm.get_field(*field);
                let addr = asm.get_node(*addr).clone();
                let addr_tpe = addr.typecheck(sig, locals, asm)?;
                let pointed_tpe = {
                    match addr_tpe {
                        Type::Ptr(type_idx) | Type::Ref(type_idx) => Some(asm[type_idx]),
                        Type::ClassRef(_) => Some(addr_tpe),
                        _ => None,
                    }
                }
                .ok_or(TypeCheckError::TypeNotPtr { tpe: addr_tpe })?;

                let Type::ClassRef(pointed_owner) = pointed_tpe else {
                    return Err(TypeCheckError::FieldAccessInvalidType {
                        tpe: pointed_tpe,
                        field,
                    });
                };
                if pointed_owner != field.owner() {
                    return Err(TypeCheckError::FieldOwnerMismatch {
                        owner: pointed_owner,
                        expected_owner: field.owner(),
                        field,
                    });
                }
                // Check that this type owns a matching field
                if let Some(cdef) = asm.class_ref_to_def(field.owner()) {
                    if !asm[cdef]
                        .fields()
                        .iter()
                        .any(|(tpe, name, _offset)| *tpe == field.tpe() && *name == field.name())
                    {
                        return Err(TypeCheckError::FieldNotPresent {
                            tpe: field.tpe(),
                            name: field.name(),
                            owner: field.owner(),
                        });
                    }
                }
                match addr_tpe {
                    Type::Ref(_) => Ok(asm.nref(field.tpe())),
                    Type::Ptr(_) => Ok(asm.nptr(field.tpe())),
                    _ => panic!("impossible. Type not a pointer or ref, but got dereferned during typechecks. {addr_tpe:?}"),
                }
            }

            CILNode::LdField { addr, field } => {
                let field = *asm.get_field(*field);
                let addr = asm.get_node(*addr).clone();
                let addr_tpe = addr.typecheck(sig, locals, asm)?;
                let pointed_tpe = {
                    match addr_tpe {
                        Type::Ptr(type_idx) | Type::Ref(type_idx) => Some(asm[type_idx]),
                        Type::ClassRef(_) => Some(addr_tpe),
                        _ => None,
                    }
                }
                .ok_or(TypeCheckError::TypeNotPtr { tpe: addr_tpe })?;
                let Type::ClassRef(pointed_owner) = pointed_tpe else {
                    return Err(TypeCheckError::FieldAccessInvalidType {
                        tpe: pointed_tpe,
                        field,
                    });
                };
                if pointed_owner != field.owner() {
                    return Err(TypeCheckError::FieldOwnerMismatch {
                        owner: pointed_owner,
                        expected_owner: field.owner(),
                        field,
                    });
                }
                // Check that this type owns a matching field
                if let Some(cdef) = asm.class_ref_to_def(field.owner()) {
                    if !asm[cdef]
                        .fields()
                        .iter()
                        .any(|(tpe, name, _offset)| *tpe == field.tpe() && *name == field.name())
                    {
                        return Err(TypeCheckError::FieldNotPresent {
                            tpe: field.tpe(),
                            name: field.name(),
                            owner: field.owner(),
                        });
                    }
                }
                Ok(field.tpe())
            }
            CILNode::LdInd {
                addr,
                tpe,
                volatile: volitale,
            } => {
                let addr = asm.get_node(*addr).clone();
                let addr_tpe = addr.typecheck(sig, locals, asm)?;
                let pointed_tpe = addr_tpe
                    .pointed_to()
                    .ok_or(TypeCheckError::TypeNotPtr { tpe: addr_tpe })?;
                let pointed_tpe = asm[pointed_tpe];
                let tpe = asm[*tpe];
                if !pointed_tpe.is_assignable_to(tpe, asm) {
                    Err(TypeCheckError::DerfWrongPtr {
                        expected: tpe,
                        got: pointed_tpe,
                    })
                } else {
                    Ok(pointed_tpe)
                }
            }
            CILNode::SizeOf(tpe) => match asm[*tpe] {
                Type::Void => Err(TypeCheckError::SizeOfVoid),
                _ => Ok(Type::Int(Int::I32)),
            },
            CILNode::GetException => Ok(Type::ClassRef(ClassRef::exception(asm))),
            CILNode::IsInst(obj, _) => {
                let obj = asm.get_node(*obj).clone();
                let _obj = obj.typecheck(sig, locals, asm)?;
                // TODO: check obj
                Ok(Type::Bool)
            }
            CILNode::CheckedCast(obj, cast_res) => {
                let obj = asm.get_node(*obj).clone();
                let _obj = obj.typecheck(sig, locals, asm)?;
                // TODO: check obj
                Ok(asm[*cast_res])
            }

            CILNode::LocAlloc { size } => {
                let size = asm[*size].clone().typecheck(sig, locals, asm)?;
                Ok(asm.nptr(Type::Int(Int::U8)))
            }
            CILNode::LdStaticField(sfld) => {
                let sfld = *asm.get_static_field(*sfld);
                Ok(sfld.tpe())
            }
            CILNode::LdStaticFieldAddress(sfld) => {
                let sfld = *asm.get_static_field(*sfld);
                Ok(asm.nptr(sfld.tpe()))
            }
            CILNode::LdFtn(mref) => {
                let mref = &asm[*mref];
                Ok(Type::FnPtr(mref.sig()))
            }
            CILNode::LdTypeToken(_) => Ok(Type::ClassRef(ClassRef::runtime_type_hadle(asm))),
            CILNode::LdLen(arr) => {
                let arr = asm.get_node(*arr).clone();
                let arr_tpe = arr.typecheck(sig, locals, asm)?;
                let Type::PlatformArray { elem: _, dims } = arr_tpe else {
                    return Err(TypeCheckError::LdLenArgNotArray { got: arr_tpe });
                };
                if dims.get() != 1 {
                    return Err(TypeCheckError::LdLenArrNot1D { got: arr_tpe });
                }
                Ok(Type::Int(Int::I32))
            }
            CILNode::LocAllocAlgined { tpe, align } => Ok(Type::Ptr(*tpe)),
            CILNode::LdElelemRef { array, index } => {
                let arr = asm.get_node(*array).clone();
                let arr_tpe = arr.typecheck(sig, locals, asm)?;
                let index = asm.get_node(*index).clone();
                let index_tpe = index.typecheck(sig, locals, asm)?;
                let Type::PlatformArray { elem, dims } = arr_tpe else {
                    return Err(TypeCheckError::LdLenArgNotArray { got: arr_tpe });
                };
                if dims.get() != 1 {
                    return Err(TypeCheckError::LdLenArrNot1D { got: arr_tpe });
                }
                match index_tpe {
                    Type::Int(Int::I32 | Int::U32 | Int::I64 | Int::USize | Int::ISize) => (),
                    _ => return Err(TypeCheckError::ArrIndexInvalidType { index_tpe }),
                }
                Ok(asm[elem])
            }
            CILNode::UnboxAny { object, tpe } => {
                let object = asm.get_node(*object).clone();
                let object = object.typecheck(sig, locals, asm)?;
                match object {
                    Type::ClassRef(cref) => {
                        let cref = asm.class_ref(cref);
                        if cref.is_valuetype() {
                            return Err(TypeCheckError::ExpectedClassGotValuetype {
                                cref: cref.clone(),
                            });
                        }
                    }
                    Type::PlatformObject | Type::PlatformGeneric(_, _) | Type::PlatformString => (),
                    _ => return Err(TypeCheckError::TypeNotClass { object }),
                };
                Ok(asm[*tpe])
            }
        }
    }
}
impl CILRoot {
    pub fn typecheck(
        &self,
        sig: Interned<FnSig>,
        locals: &[LocalDef],
        asm: &mut Assembly,
    ) -> Result<(), TypeCheckError> {
        match self {
            Self::StLoc(loc, node) => {
                let got = asm.get_node(*node).clone().typecheck(sig, locals, asm)?;
                let expected = asm[locals[*loc as usize].1];
                if !got.is_assignable_to(expected, asm) {
                    Err(TypeCheckError::LocalAssigementWrong {
                        loc: *loc,
                        got: got.mangle(asm),
                        expected: expected.mangle(asm),
                    })
                } else {
                    Ok(())
                }
            }
            Self::Branch(boxed) => {
                let (_, _, cond) = boxed.as_ref();
                let Some(cond) = cond else { return Ok(()) };
                match cond {
                    super::BranchCond::True(cond) | super::BranchCond::False(cond) => {
                        let cond = asm[*cond].clone().typecheck(sig, locals, asm)?;
                        match cond {
                            Type::Bool => Ok(()),
                            Type::Int(_) => Ok(()),
                            _ => Err(TypeCheckError::ConditionNotBool { cond }),
                        }
                    }
                    super::BranchCond::Eq(lhs, rhs)
                    | super::BranchCond::Ne(lhs, rhs)
                    | super::BranchCond::Lt(lhs, rhs, _)
                    | super::BranchCond::Gt(lhs, rhs, _)
                    | super::BranchCond::Le(lhs, rhs, _)
                    | super::BranchCond::Ge(lhs, rhs, _) => {
                        let lhs = asm[*lhs].clone().typecheck(sig, locals, asm)?;
                        let rhs = asm[*rhs].clone().typecheck(sig, locals, asm)?;
                        if lhs.is_assignable_to(rhs, asm)
                            && lhs
                                .as_class_ref()
                                .is_none_or(|cref| !asm[cref].is_valuetype())
                        {
                            Ok(())
                        } else {
                            Err(TypeCheckError::CantCompareTypes { lhs, rhs })
                        }
                    }
                }
            }
            Self::StInd(boxed) => {
                let (addr, value, tpe, _) = boxed.as_ref();
                let addr = asm[*addr].clone().typecheck(sig, locals, asm)?;
                let value = asm[*value].clone().typecheck(sig, locals, asm)?;
                let Some(addr_points_to) = addr.pointed_to().map(|tpe| asm[tpe]) else {
                    return Err(TypeCheckError::WriteWrongAddr {
                        addr: addr.mangle(asm),
                        tpe: tpe.mangle(asm),
                    });
                };
                if !(tpe.is_assignable_to(addr_points_to, asm)
                    || addr_points_to
                        .as_int()
                        .zip(tpe.as_int())
                        .is_some_and(|(a, b)| a.as_unsigned() == b.as_unsigned())
                    || addr_points_to == Type::Bool && *tpe == Type::Int(Int::I8))
                {
                    return Err(TypeCheckError::WriteWrongAddr {
                        addr: addr.mangle(asm),
                        tpe: tpe.mangle(asm),
                    });
                }
                if !(value.is_assignable_to(*tpe, asm)
                    || value
                        .as_int()
                        .zip(tpe.as_int())
                        .is_some_and(|(a, b)| a.as_unsigned() == b.as_unsigned())
                    || value == Type::Bool && *tpe == Type::Int(Int::I8))
                {
                    return Err(TypeCheckError::WriteWrongValue { tpe: *tpe, value });
                }
                Ok(())
            }
            Self::SetField(boxed) => {
                let (fld, addr, val) = boxed.as_ref();
                let addr = asm[*addr].clone().typecheck(sig, locals, asm)?;
                let val: Type = asm[*val].clone().typecheck(sig, locals, asm)?;
                let field = asm[*fld];
                let field_tpe = field.tpe();
                if !val.is_assignable_to(field_tpe, asm) {
                    return Err(TypeCheckError::FieldAssignWrongType {
                        field_tpe,
                        fld: *fld,
                        val,
                    });
                }
                let Some(pointed_tpe) = addr.pointed_to().map(|tpe| asm[tpe]) else {
                    return Err(TypeCheckError::TypeNotPtr { tpe: addr });
                };
                let Type::ClassRef(pointed_owner) = pointed_tpe else {
                    return Err(TypeCheckError::FieldAccessInvalidType {
                        tpe: pointed_tpe,
                        field,
                    });
                };
                if pointed_owner != field.owner() {
                    return Err(TypeCheckError::FieldOwnerMismatch {
                        owner: pointed_owner,
                        expected_owner: field.owner(),
                        field,
                    });
                }
                // Check that this type owns a matching field
                if let Some(cdef) = asm.class_ref_to_def(field.owner()) {
                    if !asm[cdef]
                        .fields()
                        .iter()
                        .any(|(tpe, name, _offset)| *tpe == field.tpe() && *name == field.name())
                    {
                        return Err(TypeCheckError::FieldNotPresent {
                            tpe: field.tpe(),
                            name: field.name(),
                            owner: field.owner(),
                        });
                    }
                }
                Ok(())
            }
            Self::Call(boxed) => {
                let (mref, args, _is_pure) = boxed.as_ref();
                let mref = asm[*mref].clone();
                let call_sig = asm[mref.sig()].clone();
                match mref.kind() {
                    crate::cilnode::MethodKind::Static => {
                        let expected = call_sig.inputs().len();
                        let got = args.len();
                        if expected != got {
                            return Err(TypeCheckError::CallArgcWrong {
                                expected,
                                got,
                                mname: asm[mref.name()].into(),
                            });
                        }
                    }
                    crate::cilnode::MethodKind::Instance
                    | crate::cilnode::MethodKind::Virtual
                    | crate::cilnode::MethodKind::Constructor => (),
                }
                for (index, (arg, expected)) in
                    args.iter().zip(call_sig.inputs().iter()).enumerate()
                {
                    let arg = asm[*arg].clone().typecheck(sig, locals, asm)?;
                    if !arg.is_assignable_to(*expected, asm) {
                        return Err(TypeCheckError::CallArgTypeWrong {
                            got: arg.mangle(asm),
                            expected: expected.mangle(asm),
                            idx: index,
                            mname: asm[mref.name()].into(),
                        });
                    }
                }
                Ok(())
            }
            _ => {
                for node in self.nodes() {
                    asm.get_node(*node).clone().typecheck(sig, locals, asm)?;
                }
                Ok(())
            }
        }
    }
}
#[test]
fn test() {
    let mut asm = Assembly::default();
    let lhs = super::Const::I64(0);
    let rhs = super::Const::F64(super::hashable::HashableF64(0.0));
    asm.biop(lhs, rhs, BinOp::Add);
    let _sig = asm.sig([], Type::Void);
}
