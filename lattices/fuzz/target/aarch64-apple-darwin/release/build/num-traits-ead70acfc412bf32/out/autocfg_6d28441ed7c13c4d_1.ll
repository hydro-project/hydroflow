; ModuleID = 'autocfg_6d28441ed7c13c4d_1.d5150f59c3845a69-cgu.0'
source_filename = "autocfg_6d28441ed7c13c4d_1.d5150f59c3845a69-cgu.0"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx11.0.0"

@alloc_f93507f8ba4b5780b14b2c2584609be0 = internal constant { <{ [8 x i8] }>, [24 x i8] } { <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\F0?" }>, [24 x i8] zeroinitializer }, align 32
@alloc_ef0a1f828f3393ef691f2705e817091c = internal constant { <{ [8 x i8] }>, [24 x i8] } { <{ [8 x i8] }> <{ [8 x i8] c"\00\00\00\00\00\00\00@" }>, [24 x i8] zeroinitializer }, align 32
@__asan_shadow_memory_dynamic_address = external global i64
@___asan_gen_ = private constant [50 x i8] c"autocfg_6d28441ed7c13c4d_1.d5150f59c3845a69-cgu.0\00", align 1
@___asan_gen_.1 = private unnamed_addr constant [39 x i8] c"alloc_f93507f8ba4b5780b14b2c2584609be0\00", align 1
@___asan_gen_.2 = private unnamed_addr constant [39 x i8] c"alloc_ef0a1f828f3393ef691f2705e817091c\00", align 1
@__asan_global_alloc_f93507f8ba4b5780b14b2c2584609be0 = internal global { i64, i64, i64, i64, i64, i64, i64, i64 } { i64 ptrtoint (ptr @0 to i64), i64 8, i64 32, i64 ptrtoint (ptr @___asan_gen_.1 to i64), i64 ptrtoint (ptr @___asan_gen_ to i64), i64 0, i64 0, i64 -1 }, section "__DATA,__asan_globals,regular"
@__asan_binder_alloc_f93507f8ba4b5780b14b2c2584609be0 = internal global { i64, i64 } { i64 ptrtoint (ptr @0 to i64), i64 ptrtoint (ptr @__asan_global_alloc_f93507f8ba4b5780b14b2c2584609be0 to i64) }, section "__DATA,__asan_liveness,regular,live_support"
@__asan_global_alloc_ef0a1f828f3393ef691f2705e817091c = internal global { i64, i64, i64, i64, i64, i64, i64, i64 } { i64 ptrtoint (ptr @1 to i64), i64 8, i64 32, i64 ptrtoint (ptr @___asan_gen_.2 to i64), i64 ptrtoint (ptr @___asan_gen_ to i64), i64 0, i64 0, i64 -1 }, section "__DATA,__asan_globals,regular"
@__asan_binder_alloc_ef0a1f828f3393ef691f2705e817091c = internal global { i64, i64 } { i64 ptrtoint (ptr @1 to i64), i64 ptrtoint (ptr @__asan_global_alloc_ef0a1f828f3393ef691f2705e817091c to i64) }, section "__DATA,__asan_liveness,regular,live_support"
@___asan_globals_registered = common hidden global i64 0
@llvm.global_dtors = appending global [1 x { i32, ptr, ptr }] [{ i32, ptr, ptr } { i32 1, ptr @asan.module_dtor, ptr null }]
@__sancov_lowest_stack = external thread_local(initialexec) global i64
@__sancov_gen_ = private global [4 x i8] zeroinitializer, section "__DATA,__sancov_cntrs", align 1
@__sancov_gen_.3 = private constant [8 x ptr] [ptr @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE", ptr inttoptr (i64 1 to ptr), ptr blockaddress(@"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE", %9), ptr null, ptr blockaddress(@"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE", %19), ptr null, ptr blockaddress(@"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE", %22), ptr null], section "__DATA,__sancov_pcs", align 8
@__sancov_gen_.4 = private global [1 x i8] zeroinitializer, section "__DATA,__sancov_cntrs", align 1
@__sancov_gen_.5 = private constant [2 x ptr] [ptr @_ZN26autocfg_6d28441ed7c13c4d_15probe17h369924a976b1fdc4E, ptr inttoptr (i64 1 to ptr)], section "__DATA,__sancov_pcs", align 8
@__sancov_gen_.6 = private global [1 x i8] zeroinitializer, section "__DATA,__sancov_cntrs", align 1
@__sancov_gen_.7 = private constant [2 x ptr] [ptr @asan.module_dtor, ptr inttoptr (i64 1 to ptr)], section "__DATA,__sancov_pcs", align 8
@"\01section$start$__DATA$__sancov_cntrs" = extern_weak hidden global i8
@"\01section$end$__DATA$__sancov_cntrs" = extern_weak hidden global i8
@llvm.global_ctors = appending global [2 x { i32, ptr, ptr }] [{ i32, ptr, ptr } { i32 1, ptr @asan.module_ctor, ptr null }, { i32, ptr, ptr } { i32 2, ptr @sancov.module_ctor_8bit_counters, ptr null }]
@"\01section$start$__DATA$__sancov_pcs" = extern_weak hidden global i64
@"\01section$end$__DATA$__sancov_pcs" = extern_weak hidden global i64
@llvm.used = appending global [9 x ptr] [ptr @asan.module_ctor, ptr @asan.module_dtor, ptr @sancov.module_ctor_8bit_counters, ptr @__sancov_gen_, ptr @__sancov_gen_.3, ptr @__sancov_gen_.4, ptr @__sancov_gen_.5, ptr @__sancov_gen_.6, ptr @__sancov_gen_.7], section "llvm.metadata"
@llvm.compiler.used = appending global [4 x ptr] [ptr @alloc_f93507f8ba4b5780b14b2c2584609be0, ptr @alloc_ef0a1f828f3393ef691f2705e817091c, ptr @__asan_binder_alloc_f93507f8ba4b5780b14b2c2584609be0, ptr @__asan_binder_alloc_ef0a1f828f3393ef691f2705e817091c], section "llvm.metadata"

@0 = private alias { <{ [8 x i8] }>, [24 x i8] }, ptr @alloc_f93507f8ba4b5780b14b2c2584609be0
@1 = private alias { <{ [8 x i8] }>, [24 x i8] }, ptr @alloc_ef0a1f828f3393ef691f2705e817091c

; core::f64::<impl f64>::total_cmp
; Function Attrs: inlinehint sanitize_address uwtable
define hidden i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE"(ptr align 8 %self, ptr align 8 %other) unnamed_addr #0 {
start:
  %right = alloca i64, align 8
  %left = alloca i64, align 8
  %0 = load i8, ptr @__sancov_gen_, align 1, !nosanitize !2
  %1 = add i8 %0, 1
  store i8 %1, ptr @__sancov_gen_, align 1, !nosanitize !2
  %2 = load i64, ptr @__asan_shadow_memory_dynamic_address, align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %left)
  %3 = ptrtoint ptr %self to i64
  %4 = lshr i64 %3, 3
  %5 = add i64 %4, %2
  %6 = inttoptr i64 %5 to ptr
  %7 = load i8, ptr %6, align 1
  call void @__sanitizer_cov_trace_const_cmp1(i8 0, i8 %7)
  %8 = icmp ne i8 %7, 0
  br i1 %8, label %9, label %12

9:                                                ; preds = %start
  %10 = load i8, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 1), align 1, !nosanitize !2
  %11 = add i8 %10, 1
  store i8 %11, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 1), align 1, !nosanitize !2
  call void @__asan_report_load8(i64 %3) #5
  unreachable

12:                                               ; preds = %start
  %self1 = load double, ptr %self, align 8
  %_4 = bitcast double %self1 to i64
  store i64 %_4, ptr %left, align 8
  call void @llvm.lifetime.start.p0(i64 8, ptr %right)
  %13 = ptrtoint ptr %other to i64
  %14 = lshr i64 %13, 3
  %15 = add i64 %14, %2
  %16 = inttoptr i64 %15 to ptr
  %17 = load i8, ptr %16, align 1
  call void @__sanitizer_cov_trace_const_cmp1(i8 0, i8 %17)
  %18 = icmp ne i8 %17, 0
  br i1 %18, label %19, label %22

19:                                               ; preds = %12
  %20 = load i8, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 2), align 1, !nosanitize !2
  %21 = add i8 %20, 1
  store i8 %21, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 2), align 1, !nosanitize !2
  call void @__asan_report_load8(i64 %13) #5
  unreachable

22:                                               ; preds = %12
  %23 = load i8, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 3), align 1, !nosanitize !2
  %24 = add i8 %23, 1
  store i8 %24, ptr getelementptr inbounds ([4 x i8], ptr @__sancov_gen_, i64 0, i64 3), align 1, !nosanitize !2
  %self2 = load double, ptr %other, align 8
  %_7 = bitcast double %self2 to i64
  store i64 %_7, ptr %right, align 8
  %_13 = load i64, ptr %left, align 8
  %_12 = ashr i64 %_13, 63
  %_10 = lshr i64 %_12, 1
  %25 = load i64, ptr %left, align 8
  %26 = xor i64 %25, %_10
  store i64 %26, ptr %left, align 8
  %_18 = load i64, ptr %right, align 8
  %_17 = ashr i64 %_18, 63
  %_15 = lshr i64 %_17, 1
  %27 = load i64, ptr %right, align 8
  %28 = xor i64 %27, %_15
  store i64 %28, ptr %right, align 8
  %_21 = load i64, ptr %left, align 8
  %_22 = load i64, ptr %right, align 8
  call void @__sanitizer_cov_trace_cmp8(i64 %_21, i64 %_22)
  %29 = icmp sgt i64 %_21, %_22
  %30 = zext i1 %29 to i8
  call void @__sanitizer_cov_trace_cmp8(i64 %_21, i64 %_22)
  %31 = icmp slt i64 %_21, %_22
  %32 = zext i1 %31 to i8
  %_0 = sub nsw i8 %30, %32
  call void @llvm.lifetime.end.p0(i64 8, ptr %right)
  call void @llvm.lifetime.end.p0(i64 8, ptr %left)
  ret i8 %_0
}

; autocfg_6d28441ed7c13c4d_1::probe
; Function Attrs: sanitize_address uwtable
define void @_ZN26autocfg_6d28441ed7c13c4d_15probe17h369924a976b1fdc4E() unnamed_addr #1 {
start:
  %0 = load i8, ptr @__sancov_gen_.4, align 1, !nosanitize !2
  %1 = add i8 %0, 1
  store i8 %1, ptr @__sancov_gen_.4, align 1, !nosanitize !2
  %2 = load i64, ptr @__asan_shadow_memory_dynamic_address, align 8
; call core::f64::<impl f64>::total_cmp
  %_1 = call i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$9total_cmp17h7d4a70cea857fa0aE"(ptr align 8 @alloc_f93507f8ba4b5780b14b2c2584609be0, ptr align 8 @alloc_ef0a1f828f3393ef691f2705e817091c)
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr nocapture) #2

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr nocapture) #2

declare void @__asan_report_load_n(i64, i64)

declare void @__asan_loadN(i64, i64)

declare void @__asan_report_load1(i64)

declare void @__asan_load1(i64)

declare void @__asan_report_load2(i64)

declare void @__asan_load2(i64)

declare void @__asan_report_load4(i64)

declare void @__asan_load4(i64)

declare void @__asan_report_load8(i64)

declare void @__asan_load8(i64)

declare void @__asan_report_load16(i64)

declare void @__asan_load16(i64)

declare void @__asan_report_store_n(i64, i64)

declare void @__asan_storeN(i64, i64)

declare void @__asan_report_store1(i64)

declare void @__asan_store1(i64)

declare void @__asan_report_store2(i64)

declare void @__asan_store2(i64)

declare void @__asan_report_store4(i64)

declare void @__asan_store4(i64)

declare void @__asan_report_store8(i64)

declare void @__asan_store8(i64)

declare void @__asan_report_store16(i64)

declare void @__asan_store16(i64)

declare void @__asan_report_exp_load_n(i64, i64, i32)

declare void @__asan_exp_loadN(i64, i64, i32)

declare void @__asan_report_exp_load1(i64, i32)

declare void @__asan_exp_load1(i64, i32)

declare void @__asan_report_exp_load2(i64, i32)

declare void @__asan_exp_load2(i64, i32)

declare void @__asan_report_exp_load4(i64, i32)

declare void @__asan_exp_load4(i64, i32)

declare void @__asan_report_exp_load8(i64, i32)

declare void @__asan_exp_load8(i64, i32)

declare void @__asan_report_exp_load16(i64, i32)

declare void @__asan_exp_load16(i64, i32)

declare void @__asan_report_exp_store_n(i64, i64, i32)

declare void @__asan_exp_storeN(i64, i64, i32)

declare void @__asan_report_exp_store1(i64, i32)

declare void @__asan_exp_store1(i64, i32)

declare void @__asan_report_exp_store2(i64, i32)

declare void @__asan_exp_store2(i64, i32)

declare void @__asan_report_exp_store4(i64, i32)

declare void @__asan_exp_store4(i64, i32)

declare void @__asan_report_exp_store8(i64, i32)

declare void @__asan_exp_store8(i64, i32)

declare void @__asan_report_exp_store16(i64, i32)

declare void @__asan_exp_store16(i64, i32)

declare ptr @__asan_memmove(ptr, ptr, i64)

declare ptr @__asan_memcpy(ptr, ptr, i64)

declare ptr @__asan_memset(ptr, i32, i64)

declare void @__asan_handle_no_return()

declare void @__sanitizer_ptr_cmp(i64, i64)

declare void @__sanitizer_ptr_sub(i64, i64)

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i1 @llvm.amdgcn.is.shared(ptr nocapture) #3

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i1 @llvm.amdgcn.is.private(ptr nocapture) #3

declare void @__asan_before_dynamic_init(i64)

declare void @__asan_after_dynamic_init()

declare void @__asan_register_globals(i64, i64)

declare void @__asan_unregister_globals(i64, i64)

declare void @__asan_register_image_globals(i64)

declare void @__asan_unregister_image_globals(i64)

declare void @__asan_register_elf_globals(i64, i64, i64)

declare void @__asan_unregister_elf_globals(i64, i64, i64)

declare void @__asan_init()

; Function Attrs: nounwind
define internal void @asan.module_ctor() #4 {
  call void @__asan_init()
  call void @__asan_version_mismatch_check_v8()
  call void @__asan_register_image_globals(i64 ptrtoint (ptr @___asan_globals_registered to i64))
  ret void
}

declare void @__asan_version_mismatch_check_v8()

; Function Attrs: nounwind
define internal void @asan.module_dtor() #4 {
  %1 = load i8, ptr @__sancov_gen_.6, align 1, !nosanitize !2
  %2 = add i8 %1, 1
  store i8 %2, ptr @__sancov_gen_.6, align 1, !nosanitize !2
  call void @__asan_unregister_image_globals(i64 ptrtoint (ptr @___asan_globals_registered to i64))
  ret void
}

declare void @__sanitizer_cov_trace_pc_indir(i64)

declare void @__sanitizer_cov_trace_cmp1(i8 zeroext, i8 zeroext)

declare void @__sanitizer_cov_trace_cmp2(i16 zeroext, i16 zeroext)

declare void @__sanitizer_cov_trace_cmp4(i32 zeroext, i32 zeroext)

declare void @__sanitizer_cov_trace_cmp8(i64, i64)

declare void @__sanitizer_cov_trace_const_cmp1(i8 zeroext, i8 zeroext)

declare void @__sanitizer_cov_trace_const_cmp2(i16 zeroext, i16 zeroext)

declare void @__sanitizer_cov_trace_const_cmp4(i32 zeroext, i32 zeroext)

declare void @__sanitizer_cov_trace_const_cmp8(i64, i64)

declare void @__sanitizer_cov_load1(ptr)

declare void @__sanitizer_cov_load2(ptr)

declare void @__sanitizer_cov_load4(ptr)

declare void @__sanitizer_cov_load8(ptr)

declare void @__sanitizer_cov_load16(ptr)

declare void @__sanitizer_cov_store1(ptr)

declare void @__sanitizer_cov_store2(ptr)

declare void @__sanitizer_cov_store4(ptr)

declare void @__sanitizer_cov_store8(ptr)

declare void @__sanitizer_cov_store16(ptr)

declare void @__sanitizer_cov_trace_div4(i32 zeroext)

declare void @__sanitizer_cov_trace_div8(i64)

declare void @__sanitizer_cov_trace_gep(i64)

declare void @__sanitizer_cov_trace_switch(i64, ptr)

declare void @__sanitizer_cov_trace_pc()

declare void @__sanitizer_cov_trace_pc_guard(ptr)

declare void @__sanitizer_cov_8bit_counters_init(ptr, ptr)

; Function Attrs: nounwind
define internal void @sancov.module_ctor_8bit_counters() #4 {
  call void @__sanitizer_cov_8bit_counters_init(ptr @"\01section$start$__DATA$__sancov_cntrs", ptr @"\01section$end$__DATA$__sancov_cntrs")
  call void @__sanitizer_cov_pcs_init(ptr @"\01section$start$__DATA$__sancov_pcs", ptr @"\01section$end$__DATA$__sancov_pcs")
  ret void
}

declare void @__sanitizer_cov_pcs_init(ptr, ptr)

attributes #0 = { inlinehint sanitize_address uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #1 = { sanitize_address uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #2 = { nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #4 = { nounwind }
attributes #5 = { nomerge }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.79.0-nightly (8b2459c1f 2024-04-09)"}
!2 = !{}
