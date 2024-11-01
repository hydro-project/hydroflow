,
                Some(|a: $type, b: $type| a.wrapping_mul(b)),
                Some(|a: $type| a)
            )
        });
    };
}
