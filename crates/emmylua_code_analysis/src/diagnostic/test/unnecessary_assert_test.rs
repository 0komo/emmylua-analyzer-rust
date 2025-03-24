#[cfg(test)]
mod test {
    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test_1() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::UnnecessaryAssert,
            r#"
            local a
            assert(a)

            ---@type boolean
            local b
            assert(b)

            ---@type any
            local c
            assert(c)

            ---@type unknown
            local d
            assert(d)

            ---@type boolean
            local e
            assert(e)

            ---@type number?
            local f
            assert(f)

            assert(false)

            assert(nil and 5)
        "#
        ));
    }

    #[test]
    fn test_2() {
        let mut ws = VirtualWorkspace::new();

        assert!(!ws.check_code_for(
            DiagnosticCode::UnnecessaryAssert,
            r#"
            assert(true)
        "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::UnnecessaryAssert,
            r#"
            ---@return integer
            local function hi()
              return 1
            end
            assert(hi(1))
            "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::UnnecessaryAssert,
            r#"
            assert({}, 'hi')
            "#
        ));
    }
}
