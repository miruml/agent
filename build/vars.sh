#shellcheck disable=all
version="0.6.5"
deb_pkg_name="miru"

miru_binaries_dir="/usr/local/miru"

agent_bin_name="miru-$version"
agent_bin_path="$miru_binaries_dir/$agent_bin_name"
agent_bin_symlink_name="miru"
agent_bin_symlink_path="$miru_binaries_dir/$agent_bin_symlink_name"
agent_start_script_name="miru-start.sh"

install_bin_name="install-miru"
install_bin_path="$miru_binaries_dir/$install_bin_name"

systemd_dir="/etc/systemd/system"
systemd_service_name="miru.service"
systemd_service_path="$systemd_dir/$systemd_service_name"

miru_dir="/var/lib/miru"
miru_log_dir="/var/log/miru"