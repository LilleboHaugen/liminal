// TODO(M0-T005): tracked, open task
// TODO: untracked
/// FIXME doc comment
//! HACK inner doc comment
/* XXX block comment */
/*
 * TODO(M0-T001): names a done task
 */
// TODOs remain
// TODO(M0-T5) malformed task ID
// TODO (M0-T005) space before the task
// TODO(M9-T999) unknown task
// FIXME(M0-T005) HACK(M0-T005) XXX(M0-T005) all tracked
// TODOLIST MYTODO todo Todo XXXL _TODO TODO_X are not marker words
const S: &str = "TODO: in a string";
// TODO(M0-T003): task with broken frontmatter exists
fn f() {} // HACK: trailing comment
