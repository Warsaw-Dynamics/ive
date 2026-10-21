use super::keys_at;

#[test]
fn should_tell_solidity_function_declaration_from_calls_and_builtins() {
    // given
    let source = "contract C {\n  function run(uint256 n) public { check(n); token.transfer(msg.sender, n); }\n}";

    // when
    let keys = keys_at("Solidity", source, ["run", "uint256", "n)", "check", "transfer", "msg"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SOL.FUNCTION_DECLARATION"),
            Some("SOL.PRIMITIVE_TYPE"),
            Some("SOL.PARAMETER"),
            Some("SOL.FUNCTION_CALL"),
            Some("SOL.METHOD_CALL"),
            Some("SOL.BUILTIN"),
        ]
    );
}

#[test]
fn should_mark_solidity_contract_interface_event_and_modifier() {
    // given
    let source = "interface IShop {}\ncontract Shop is IShop {\n  event Sold(uint id);\n  modifier onlyOwner() { _; }\n  function buy() external onlyOwner { emit Sold(1); }\n}";

    // when
    let keys = keys_at("Solidity", source, ["IShop", "Shop is", "Sold", "onlyOwner", "onlyOwner {", "Sold(1"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SOL.INTERFACE"),
            Some("SOL.TYPE"),
            Some("SOL.EVENT"),
            Some("SOL.MODIFIER"),
            Some("SOL.MODIFIER"),
            Some("SOL.EVENT"),
        ]
    );
}

#[test]
fn should_tell_solidity_state_variable_from_constant_local_and_field() {
    // given
    let source = "contract C {\n  struct S { uint size; }\n  uint256 constant LIMIT = 3;\n  uint256 count;\n  function f() public { S memory s; s.size = LIMIT; }\n}";

    // when
    let keys = keys_at("Solidity", source, ["size", "LIMIT", "count", "s;", "constant"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SOL.FIELD"),
            Some("SOL.CONSTANT"),
            Some("SOL.STATE_VARIABLE"),
            Some("SOL.LOCAL_VARIABLE"),
            Some("SOL.KEYWORD"),
        ]
    );
}

#[test]
fn should_tell_solidity_natspec_from_plain_comments_and_pragma() {
    // given
    let source = "pragma solidity ^0.8.0;\n/// @notice doc\n/** @dev block doc */\n/* block */\n// line\ncontract C {}";

    // when
    let keys = keys_at("Solidity", source, ["0.8.0", "/// @notice", "/** @dev", "/* block", "// line"]);

    // then
    assert_eq!(
        keys,
        [
            Some("SOL.PRAGMA"),
            Some("SOL.DOC_COMMENT"),
            Some("SOL.DOC_COMMENT"),
            Some("SOL.BLOCK_COMMENT"),
            Some("SOL.LINE_COMMENT"),
        ]
    );
}
