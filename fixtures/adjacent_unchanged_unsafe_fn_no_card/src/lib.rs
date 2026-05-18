pub fn data_end(ctrl: Ptr) -> Ptr {
    ctrl.cast()
}

pub unsafe fn data_start(ptr: Ptr) -> Ptr {
    ptr
}
