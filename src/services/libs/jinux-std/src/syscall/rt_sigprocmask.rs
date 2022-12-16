use jinux_frame::vm::VmIo;

use crate::{
    log_syscall_entry,
    prelude::*,
    syscall::{SyscallReturn, SYS_RT_SIGPROCMASK},
};

pub fn sys_rt_sigprocmask(
    how: u32,
    set_ptr: Vaddr,
    oldset_ptr: Vaddr,
    sigset_size: usize,
) -> Result<SyscallReturn> {
    log_syscall_entry!(SYS_RT_SIGPROCMASK);
    let mask_op = MaskOp::try_from(how).unwrap();
    debug!(
        "mask op = {:?}, set_ptr = 0x{:x}, oldset_ptr = 0x{:x}, sigset_size = {}",
        mask_op, set_ptr, oldset_ptr, sigset_size
    );
    if sigset_size != 8 {
        error!("sigset size is not equal to 8");
    }
    do_rt_sigprocmask(mask_op, set_ptr, oldset_ptr, sigset_size).unwrap();
    Ok(SyscallReturn::Return(0))
}

fn do_rt_sigprocmask(
    mask_op: MaskOp,
    set_ptr: Vaddr,
    oldset_ptr: Vaddr,
    sigset_size: usize,
) -> Result<()> {
    let current = current!();
    let root_vmar = current.root_vmar().unwrap();
    let mut sig_mask = current.sig_mask().lock();
    let old_sig_mask_value = sig_mask.as_u64();
    debug!("old sig mask value: 0x{:x}", old_sig_mask_value);
    if oldset_ptr != 0 {
        root_vmar.write_val(oldset_ptr, &old_sig_mask_value)?;
    }
    if set_ptr != 0 {
        let new_set = root_vmar.read_val::<u64>(set_ptr)?;
        debug!("new set = 0x{:x}", new_set);
        match mask_op {
            MaskOp::Block => sig_mask.block(new_set),
            MaskOp::Unblock => sig_mask.unblock(new_set),
            MaskOp::SetMask => sig_mask.set(new_set),
        }
    }
    debug!("new set = {:x?}", &sig_mask);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MaskOp {
    Block = 0,
    Unblock = 1,
    SetMask = 2,
}

impl TryFrom<u32> for MaskOp {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        let op = match value {
            0 => MaskOp::Block,
            1 => MaskOp::Unblock,
            2 => MaskOp::SetMask,
            _ => return_errno_with_message!(Errno::EINVAL, "invalid mask op"),
        };
        Ok(op)
    }
}
