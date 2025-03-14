#[cfg(test)]
mod tests {
    use crate::{DiagnosticCode, VirtualWorkspace};

    //     /// 暂时无法解决的测试
    //     #[test]
    //     fn test_error() {
    //         let mut ws = VirtualWorkspace::new();

    //         // 推断类型异常
    //         assert!(ws.check_code_for_namespace(
    //             DiagnosticCode::AssignTypeMismatch,
    //             r#"
    // local n

    // if G then
    //     n = {}
    // else
    //     n = nil
    // end

    // local t = {
    //     x = n,
    // }
    //             "#
    //         ));

    //         // 类型匹配没有解决类继承的情况
    //         assert!(ws.check_code_for_namespace(
    //             DiagnosticCode::AssignTypeMismatch,
    //             r#"
    // ---@class Option: string

    // ---@param x Option
    // local function f(x) end

    // ---@type Option
    // local x = 'aaa'

    // f(x)
    //             "#
    //         ));

    //         // 类型匹配没有解决类继承的情况
    //         assert!(ws.check_code_for_namespace(
    //             DiagnosticCode::AssignTypeMismatch,
    //             r#"
    // ---@class A
    // local a = {}

    // ---@class B: A
    // local b = a
    //             "#
    //         ));

    //         // 数组类型匹配中不允许可空, 但luals中允许的
    //         assert!(ws.check_code_for_namespace(
    //             DiagnosticCode::AssignTypeMismatch,
    //             r#"
    // ---@type boolean[]
    // local t = { true, false, nil }
    //             "#
    //         ));
    //     }

    #[test]
    fn test_valid_cases() {
        let mut ws = VirtualWorkspace::new();

        // Test cases that should pass (no type mismatch)
        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
local m = {}
---@type integer[]
m.ints = {}
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
---@field x A

---@type A
local t

t.x = {}
            "#
        ));

        // Test cases that should fail (type mismatch)
        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
---@field x integer

---@type A
local t

t.x = true
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
---@field x integer

---@type A
local t

---@type boolean
local y

t.x = y
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local m

m.x = 1

---@type A
local t

t.x = true
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local m

---@type integer
m.x = 1

m.x = true
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local mt

---@type integer
mt.x = 1

function mt:init()
    self.x = true
end
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
---@field x integer

---@type A
local t = {
    x = true
}
            "#
        ));

        let mut ws = VirtualWorkspace::new();
        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type boolean[]
local t = {}

t[5] = nil
            "#
        ));
        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type table<string, true>
local t = {}

t['x'] = nil
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type [boolean]
local t = { [1] = nil }

t = nil
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
local t = { true }

t[1] = nil
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local t = {
    x = 1
}

t.x = true
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type number
local t

t = 1
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type number
local t

---@type integer
local y

t = y
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local m

---@type number
m.x = 1

m.x = {}
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type boolean[]
local t = {}

---@type boolean?
local x

t[#t+1] = x
            "#
        ));

        // Additional test cases
        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type number
local n
---@type integer
local i

i = n
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type number|boolean
local nb

---@type number
local n

n = nb
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type number
local x = 'aaa'
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class X

---@class A
local mt = G

---@type X
mt._x = nil
            "#
        ));
        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local a = {}

---@class B
local b = a
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
local a = {}
a.__index = a

---@class B: A
local b = setmetatable({}, a)
            "#
        ));

        // Continue with more test cases as needed
        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class A
---@field x number?
local a

---@class B
---@field x number
local b

b.x = a.x
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
local mt = {}
mt.x = 1
mt.x = nil
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@alias test boolean

---@type test
local test = 4
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class MyClass
local MyClass = {}

function MyClass:new()
    ---@class MyClass
    local myObject = setmetatable({
        initialField = true
    }, self)

    print(myObject.initialField)
end
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@class T
local t = {
    x = nil
}

t.x = 1
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type {[1]: string, [10]: number, xx: boolean}
local t = {
    true,
    [10] = 's',
    xx = 1,
}
            "#
        ));

        assert!(!ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
---@type boolean[]
local t = { 1, 2, 3 }
            "#
        ));

        assert!(ws.check_code_for_namespace(
            DiagnosticCode::AssignTypeMismatch,
            r#"
local t = {}
t.a = 1
t.a = 2
return t
            "#
        ));
    }
}
