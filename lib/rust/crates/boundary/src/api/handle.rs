//! A class that owns a native handle: it makes the handle, frees it through `RustLibrary`'s
//! cleaner once nothing can reach the object (or on `close()`), and passes it to each member.

use super::{Api, Handle, constants, member_text, method, params_and_args, push_block};
use crate::Class;
use crate::format::{KOTLIN_WIDTH, kotlin_declaration};

/// A handle class's head, handle and freeing, filling `body` with its members.
pub(super) fn handle_class(
    api: &Api,
    handle: Handle,
    classes: &[Class],
    apis: &[Api],
    body: &mut String,
) -> String {
    let free = method(classes, api.native, handle.free);
    assert_eq!(free.params.len(), 1, "{} frees with one handle", api.name);
    let field = if handle.close { "var" } else { "val" };
    let close = if handle.close { " : AutoCloseable" } else { "" };
    let (mut head, create) = if handle.create.is_empty() {
        let head = format!(
            "class {} internal constructor(handle: Long){close} {{\n",
            api.name
        );
        (head, "handle".to_owned())
    } else {
        let create = method(classes, api.native, handle.create);
        let (params, passed) = params_and_args(create, handle.params, false);
        let head = kotlin_declaration(
            "",
            &format!("class {}", api.name),
            &params,
            &format!("{close} {{"),
        );
        (head, create_call(api.native, handle.create, &passed))
    };
    head.push_str(&format!("    internal {field} handle = {create}\n"));
    if handle.close {
        head.push_str("        private set\n");
        head.push_str(
            "    private val cleanable = RustLibrary.cleaner.register(this, Free(handle))\n",
        );
    } else {
        head.push_str(
            "\n    init {\n        RustLibrary.cleaner.register(this, Free(handle))\n    }\n",
        );
    }
    let mut companion = String::new();
    for member in api.members {
        if member.companion {
            let text = member_text(api, member, Some(handle), classes, apis, "        ");
            push_block(&mut companion, &text);
        } else {
            push_block(
                body,
                &member_text(api, member, Some(handle), classes, apis, "    "),
            );
        }
    }
    if handle.lend {
        assert!(
            handle.synchronized,
            "{}: a lent handle needs the lock",
            api.name
        );
        push_block(
            body,
            "    /** Lends the handle to [action], with `close()` kept out until it returns. */\n    \
             @Synchronized\n    internal fun <T> withHandle(action: (Long) -> T): T = action(handle)\n",
        );
    }
    if handle.close {
        let lock = if handle.synchronized {
            "    @Synchronized\n"
        } else {
            ""
        };
        push_block(
            body,
            &format!(
                "{lock}    override fun close() {{\n        handle = 0\n        cleanable.clean()\n    }}\n"
            ),
        );
    }
    push_block(
        body,
        &format!(
            "    private class Free(private val handle: Long) : Runnable {{\n        \
             override fun run() = {}.{}(handle)\n    }}\n",
            api.native, handle.free
        ),
    );
    if !api.constants.is_empty() {
        push_block(&mut companion, &constants(api.constants, "        "));
    }
    if !companion.is_empty() || api.companion {
        let inside = if companion.is_empty() {
            String::new()
        } else {
            format!(" {{\n{companion}    }}")
        };
        push_block(body, &format!("    companion object{inside}\n"));
    }
    head
}

/// `native.create(passed)`, as the handle's initialiser: on its line if it fits, else one argument
/// to a line.
fn create_call(native: &str, create: &str, passed: &[String]) -> String {
    let one = format!("{native}.{create}({})", passed.join(", "));
    if "    internal var handle = ".len() + one.len() <= KOTLIN_WIDTH {
        return one;
    }
    let args: String = passed.iter().map(|it| format!("        {it},\n")).collect();
    format!("{native}.{create}(\n{args}    )")
}
