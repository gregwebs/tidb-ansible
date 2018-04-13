# centos configure to accept ansible commands from a control machine
FROM centos/systemd

# ntp
RUN yum -y install sudo ntp ntpstat && yum clean all
RUN systemctl enable ntpd.service

# Debugging
RUN yum install -y bind-utils && yum clean all

RUN useradd tidb \
 && echo "tidb ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/tidb

CMD ["/usr/sbin/init"]
