#' The wickra-copilot library version.
#' @return A version string.
#' @export
wkcopilot_version <- function() {
  .Call(C_wkcopilot_version)
}

#' Build a copilot from a spec JSON string.
#' @param spec_json A JSON spec string.
#' @return A `wickra_copilot` handle (an external pointer).
#' @export
wkcopilot_new <- function(spec_json) {
  .Call(C_wkcopilot_new, spec_json)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param copilot A copilot handle from [wkcopilot_new()].
#' @param cmd_json A command JSON string.
#' @return The response as a JSON string.
#' @export
wkcopilot_command <- function(copilot, cmd_json) {
  .Call(C_wkcopilot_command, copilot, cmd_json)
}
