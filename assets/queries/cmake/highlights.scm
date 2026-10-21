; IVE highlight query for CMake. Capture names are colour key ids (src/colors/keys/cmake.rs).
; For the same node a later pattern wins; otherwise the innermost node wins. Command names ignore case.

["(" ")"] @CMAKE.PARENTHESES

(quoted_argument) @CMAKE.STRING
(bracket_argument) @CMAKE.BRACKET_ARGUMENT
(escape_sequence) @CMAKE.ESCAPE
(line_comment) @CMAKE.LINE_COMMENT
(bracket_comment) @CMAKE.BLOCK_COMMENT

[
  (if) (elseif) (else) (endif) (foreach) (endforeach) (while) (endwhile) (function) (endfunction) (macro)
  (endmacro) (block) (endblock)
] @CMAKE.KEYWORD

((unquoted_argument) @CMAKE.KEYWORD_ARGUMENT
 (#any-of? @CMAKE.KEYWORD_ARGUMENT
  "PUBLIC" "PRIVATE" "INTERFACE" "REQUIRED" "QUIET" "COMPONENTS" "OPTIONAL_COMPONENTS" "CONFIG" "MODULE" "VERSION"
  "LANGUAGES" "DESCRIPTION" "HOMEPAGE_URL" "STATUS" "WARNING" "AUTHOR_WARNING" "SEND_ERROR" "FATAL_ERROR"
  "DEPRECATION" "NOTICE" "VERBOSE" "DEBUG" "TRACE" "CACHE" "BOOL" "FILEPATH" "PATH" "STRING" "INTERNAL" "FORCE"
  "PARENT_SCOPE" "ON" "OFF" "TRUE" "FALSE" "YES" "NO" "STATIC" "SHARED" "OBJECT" "ALIAS" "IMPORTED" "GLOBAL"
  "EXCLUDE_FROM_ALL" "WIN32" "MACOSX_BUNDLE" "APPEND" "PREPEND" "INSERT" "REMOVE_ITEM" "REMOVE_DUPLICATES" "LENGTH"
  "GET" "JOIN" "FIND" "SORT" "REVERSE" "TRANSFORM" "TARGET" "TARGETS" "FILES" "DIRECTORY" "PROGRAMS" "DESTINATION"
  "RUNTIME" "LIBRARY" "ARCHIVE" "INCLUDES" "EXPORT" "NAMESPACE" "COMMAND" "DEPENDS" "OUTPUT" "WORKING_DIRECTORY"
  "COMMENT" "VERBATIM" "ALL" "PROPERTIES" "PROPERTY" "NAMES" "HINTS" "PATHS" "PATH_SUFFIXES" "NO_DEFAULT_PATH"
  "GLOB" "GLOB_RECURSE" "READ" "WRITE" "COPY" "MAKE_DIRECTORY" "REGEX" "REPLACE" "MATCH" "MATCHALL" "TOLOWER"
  "TOUPPER" "SUBSTRING" "CONCAT" "EXPR" "RESULT_VARIABLE" "OUTPUT_VARIABLE" "SCOPE_FOR" "POLICIES" "VARIABLES"
  "PROPAGATE" "BEFORE" "AFTER" "SYSTEM" "FATAL_ON_MISSING_REQUIRED_PACKAGES"))
; a bare word in a condition names a variable
[
  (if_command (argument_list (argument (unquoted_argument) @CMAKE.VARIABLE)))
  (elseif_command (argument_list (argument (unquoted_argument) @CMAKE.VARIABLE)))
  (while_command (argument_list (argument (unquoted_argument) @CMAKE.VARIABLE)))
]
((unquoted_argument) @CMAKE.NUMBER
 (#match? @CMAKE.NUMBER "^[0-9]+(\\.[0-9]+)*$"))
([(if_command (argument_list (argument (unquoted_argument) @CMAKE.KEYWORD)))
  (elseif_command (argument_list (argument (unquoted_argument) @CMAKE.KEYWORD)))
  (while_command (argument_list (argument (unquoted_argument) @CMAKE.KEYWORD)))]
 (#any-of? @CMAKE.KEYWORD
  "NOT" "AND" "OR" "COMMAND" "POLICY" "TARGET" "TEST" "DEFINED" "IN_LIST" "EXISTS" "IS_NEWER_THAN" "IS_DIRECTORY"
  "IS_SYMLINK" "IS_ABSOLUTE" "MATCHES" "LESS" "GREATER" "EQUAL" "LESS_EQUAL" "GREATER_EQUAL" "STRLESS"
  "STRGREATER" "STREQUAL" "STRLESS_EQUAL" "STRGREATER_EQUAL" "VERSION_LESS" "VERSION_GREATER" "VERSION_EQUAL"
  "VERSION_LESS_EQUAL" "VERSION_GREATER_EQUAL"))
(foreach_command
  (argument_list (argument (unquoted_argument) @CMAKE.KEYWORD))
  (#any-of? @CMAKE.KEYWORD "IN" "LISTS" "ITEMS" "RANGE" "ZIP_LISTS"))

(normal_command (identifier) @CMAKE.FUNCTION_CALL)
((normal_command (identifier) @CMAKE.BUILTIN_COMMAND)
 (#match? @CMAKE.BUILTIN_COMMAND
  "(?i)^(add_compile_definitions|add_compile_options|add_custom_command|add_custom_target|add_definitions|add_dependencies|add_executable|add_library|add_link_options|add_subdirectory|add_test|aux_source_directory|break|build_command|cmake_host_system_information|cmake_language|cmake_minimum_required|cmake_parse_arguments|cmake_path|cmake_policy|configure_file|continue|create_test_sourcelist|ctest_[a-z_]+|define_property|enable_language|enable_testing|execute_process|export|file|find_file|find_library|find_package|find_path|find_program|fltk_wrap_ui|get_cmake_property|get_directory_property|get_filename_component|get_property|get_source_file_property|get_target_property|get_test_property|include|include_directories|include_external_msproject|include_guard|include_regular_expression|install|link_directories|link_libraries|list|load_cache|mark_as_advanced|math|message|option|project|remove_definitions|return|separate_arguments|set|set_directory_properties|set_property|set_source_files_properties|set_target_properties|set_tests_properties|site_name|source_group|string|target_compile_definitions|target_compile_features|target_compile_options|target_include_directories|target_link_directories|target_link_libraries|target_link_options|target_precompile_headers|target_sources|try_compile|try_run|unset|variable_watch)$"))
((normal_command (identifier) @CMAKE.KEYWORD)
 (#match? @CMAKE.KEYWORD "(?i)^(break|continue|return)$"))

(function_command (argument_list . (argument) @CMAKE.FUNCTION_DECLARATION (argument)* @CMAKE.PARAMETER))
(macro_command (argument_list . (argument) @CMAKE.FUNCTION_DECLARATION (argument)* @CMAKE.PARAMETER))
(foreach_command (argument_list . (argument) @CMAKE.VARIABLE_DECLARATION))
((normal_command (identifier) @_command (argument_list . (argument) @CMAKE.VARIABLE_DECLARATION))
 (#match? @_command "(?i)^(set|option|unset)$"))
((normal_command (identifier) @_command (argument_list . (argument) . (argument) @CMAKE.VARIABLE_DECLARATION))
 (#match? @_command "(?i)^list$"))

(variable_ref) @CMAKE.VARIABLE
(variable_ref (_ ["$" "{" "}" "ENV" "CACHE"] @CMAKE.VARIABLE))
(variable) @CMAKE.VARIABLE
