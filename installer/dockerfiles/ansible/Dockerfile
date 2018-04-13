FROM centos:centos7

COPY ansible ansible

RUN echo "===> Installing EPEL..."   && \
    yum -y \
           --exclude=openssh-* --exclude=policycoreutils* --exclude=libsemanage-* --exclude=selinux-* --exclude=iputils \
           install epel-release      && \
    yum -y update                    && \
    echo "===> Adding Ansible's prerequisites..."  && \
    yum -y install \
                gcc make which \
                python python-devel python-pip                 \
                libffi-devel openssl-devel                     \
                libxml2 libxml2-devel libxslt libxslt-devel    \
                git sudo curl                               && \
    pip install --upgrade pip  && \
    pip install --upgrade \
        pyyaml jinja2 pycrypto paramiko httplib2 jmespath   && \


    pip install --upgrade pywinrm                           && \
    echo "===> Compiling Ansible..." && \
    cd ansible                            && \
    bash -c 'source ./hacking/env-setup'  && \
    echo "===> Moving useful Ansible stuff to /opt/ansible ..."  && \
    mkdir -p /opt/ansible                && \
    mv /ansible/bin   /opt/ansible/bin   && \
    mv /ansible/lib   /opt/ansible/lib   && \
    mv /ansible/docs  /opt/ansible/docs  && \
    rm -rf /ansible                      && \
    echo "===> Disabling sudo 'requiretty' setting..."    && \
    sed -i -e 's/^\(Defaults\s*requiretty\)/#--- \1/'  /etc/sudoers  && \
    echo "===> Removing unused YUM resources..."              && \
    yum -y remove epel-release gcc git python-devel python-pip \
                  libffi-devel openssl-devel  || true         && \
    yum clean all && rm -rf /var/cache/yum

ENV PATH        /opt/ansible/bin:$PATH
ENV PYTHONPATH  /opt/ansible/lib:$PYTHONPATH
ENV MANPATH     /opt/ansible/docs/man:$MANPATH

RUN /opt/ansible/bin/ansible --version

CMD ["ansible", "--version"]
