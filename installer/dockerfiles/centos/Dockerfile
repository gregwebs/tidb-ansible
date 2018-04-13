# centos configure to accept ansible commands from a control machine
FROM centos/systemd

# Packages
RUN yum -y install sudo ntp ntpstat && yum clean all
RUN systemctl enable ntpd.service

RUN useradd tidb \
 && echo "tidb ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/tidb

CMD ["/usr/sbin/init"]
