%define debug_package %{nil}

Name: saitachain
Summary: Implementation of a saitachain node in Rust based on the Substrate framework.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: GPLv3
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

Requires: systemd, shadow-utils
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}


%prep
%setup -q


%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%post
config_file="/etc/default/saitachain"
getent group saitachain >/dev/null || groupadd -r saitachain
getent passwd saitachain >/dev/null || \
    useradd -r -g saitachain -d /home/saitachain -m -s /sbin/nologin \
    -c "User account for running saitachain as a service" saitachain
if [ ! -e "$config_file" ]; then
    echo 'saitachain_CLI_ARGS=""' > /etc/default/saitachain
fi
exit 0

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
/usr/lib/systemd/system/saitachain.service
