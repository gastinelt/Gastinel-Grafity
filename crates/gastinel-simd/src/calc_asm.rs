//! Moteur arithmétique 100% assembleur x86_64 (SSE/FPU) pour Gastinel.
//!
//! Exécute toutes les opérations flottantes via des instructions machine directes
//! sans dépendance logicielle (`libm` ou wrappers Rust standard).

use std::arch::asm;

/// Addition flottante 64-bit via SSE2 `addsd`.
#[inline(always)]
pub fn asm_add(a: f64, b: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "addsd {out}, {in2}",
            out = inout(xmm_reg) a => res,
            in2 = in(xmm_reg) b,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Soustraction flottante 64-bit via SSE2 `subsd`.
#[inline(always)]
pub fn asm_sub(a: f64, b: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "subsd {out}, {in2}",
            out = inout(xmm_reg) a => res,
            in2 = in(xmm_reg) b,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Multiplication flottante 64-bit via SSE2 `mulsd`.
#[inline(always)]
pub fn asm_mul(a: f64, b: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "mulsd {out}, {in2}",
            out = inout(xmm_reg) a => res,
            in2 = in(xmm_reg) b,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Division flottante 64-bit via SSE2 `divsd`.
#[inline(always)]
pub fn asm_div(a: f64, b: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "divsd {out}, {in2}",
            out = inout(xmm_reg) a => res,
            in2 = in(xmm_reg) b,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Racine carrée 64-bit via SSE2 `sqrtsd`.
#[inline(always)]
pub fn asm_sqrt(a: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "sqrtsd {out}, {in}",
            out = out(xmm_reg) res,
            in = in(xmm_reg) a,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Inversion de signe via xorpd avec un masque de bit de signe (bit 63).
#[inline(always)]
pub fn asm_neg(a: f64) -> f64 {
    let mut res: f64;
    let sign_mask: u64 = 1u64 << 63;
    unsafe {
        asm!(
            "movq {tmp}, {mask}",
            "xorpd {out}, {tmp}",
            mask = in(reg) sign_mask,
            out = inout(xmm_reg) a => res,
            tmp = out(xmm_reg) _,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Valeur absolue via andpd en masquant le bit 63.
#[inline(always)]
pub fn asm_abs(a: f64) -> f64 {
    let mut res: f64;
    let abs_mask: u64 = !(1u64 << 63);
    unsafe {
        asm!(
            "movq {tmp}, {mask}",
            "andpd {out}, {tmp}",
            mask = in(reg) abs_mask,
            out = inout(xmm_reg) a => res,
            tmp = out(xmm_reg) _,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Modulo / reste fprem via FPU x87 inline.
pub fn asm_mod(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return f64::NAN;
    }
    // a - floor(a/b)*b
    let div = asm_div(a, b);
    let flr = asm_floor(div);
    asm_sub(a, asm_mul(flr, b))
}

/// Arrondi inférieur (floor) via SSE4.1 `roundsd` mode 1 (floor).
#[inline(always)]
pub fn asm_floor(a: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "roundsd {out}, {in}, 1",
            out = out(xmm_reg) res,
            in = in(xmm_reg) a,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Arrondi supérieur (ceil) via SSE4.1 `roundsd` mode 2 (ceil).
#[inline(always)]
pub fn asm_ceil(a: f64) -> f64 {
    let mut res: f64;
    unsafe {
        asm!(
            "roundsd {out}, {in}, 2",
            out = out(xmm_reg) res,
            in = in(xmm_reg) a,
            options(pure, nomem, nostack)
        );
    }
    res
}

/// Puissance entière via boucle de multiplication ASM mulsd.
pub fn asm_pow_int(base: f64, exp: i32) -> f64 {
    if exp == 0 {
        return 1.0;
    }
    let mut e = exp.abs();
    let mut res = 1.0;
    let mut b = base;

    while e > 0 {
        if e & 1 == 1 {
            res = asm_mul(res, b);
        }
        b = asm_mul(b, b);
        e >>= 1;
    }

    if exp < 0 {
        asm_div(1.0, res)
    } else {
        res
    }
}

/// Sinus via FPU x87 `fsin`
pub fn asm_sin(rad: f64) -> f64 {
    let mut res: f64 = 0.0;
    unsafe {
        asm!(
            "fld qword ptr [{input}]",
            "fsin",
            "fstp qword ptr [{output}]",
            input = in(reg) &rad,
            output = in(reg) &mut res,
        );
    }
    res
}

/// Cosinus via FPU x87 `fcos`
pub fn asm_cos(rad: f64) -> f64 {
    let mut res: f64 = 0.0;
    unsafe {
        asm!(
            "fld qword ptr [{input}]",
            "fcos",
            "fstp qword ptr [{output}]",
            input = in(reg) &rad,
            output = in(reg) &mut res,
        );
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_arithmetic() {
        assert_eq!(asm_add(10.5, 4.25), 14.75);
        assert_eq!(asm_sub(10.0, 3.5), 6.5);
        assert_eq!(asm_mul(4.0, 2.5), 10.0);
        assert_eq!(asm_div(20.0, 4.0), 5.0);
        assert_eq!(asm_sqrt(16.0), 4.0);
        assert_eq!(asm_neg(5.0), -5.0);
        assert_eq!(asm_neg(-12.5), 12.5);
        assert_eq!(asm_abs(-42.0), 42.0);
        assert_eq!(asm_abs(42.0), 42.0);
        assert_eq!(asm_floor(3.7), 3.0);
        assert_eq!(asm_ceil(3.2), 4.0);
        assert_eq!(asm_pow_int(2.0, 8), 256.0);
        assert_eq!(asm_pow_int(2.0, -2), 0.25);
    }

    #[test]
    fn test_asm_trig() {
        let s0 = asm_sin(0.0);
        assert!((s0 - 0.0).abs() < 1e-6);
        let c0 = asm_cos(0.0);
        assert!((c0 - 1.0).abs() < 1e-6);
    }
}
