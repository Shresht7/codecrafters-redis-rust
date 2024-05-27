// Library
use crate::parser::resp::Type;

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub fn command(args: &[Type]) -> Type {
    // Check the number of arguments
    if args.len() < 1 {
        return Type::SimpleError(
            "ERR at least one argument is required for 'ECHO' command".into(),
        );
    }

    // Respond with the argument
    if let Some(Type::BulkString(arg)) = args.get(0) {
        Type::BulkString(arg.clone())
    } else {
        Type::SimpleError("ERR invalid argument type for 'ECHO' command".into())
    }
}
