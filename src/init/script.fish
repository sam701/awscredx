set -x AWSCREDX_SCRIPT_VERSION "@version@"

function assume
  set -l output ("@bin@" assume $argv)
  if test $status -eq 0
    eval "$output"
  end
end
