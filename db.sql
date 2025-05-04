CREATE TABLE `t_menu` (
                          `uid` bigint NOT NULL AUTO_INCREMENT COMMENT '主键',
                          `value` varchar(32) NOT NULL COMMENT '权限值',
                          `name` varchar(64) NOT NULL COMMENT '权限名称',
                          `type` varchar(16) NOT NULL COMMENT '类型。menu：菜单，button：按钮',
                          `creator_id` bigint NOT NULL COMMENT '创建人id',
                          `create_time` timestamp NOT NULL DEFAULT (now()) COMMENT '创建时间',
                          `updater_id` bigint DEFAULT NULL COMMENT '更新人id',
                          `update_time` timestamp NULL DEFAULT NULL COMMENT '更新时间',
                          PRIMARY KEY (`uid`),
                          UNIQUE KEY `t_menu_value_uindex` (`value`)
) ENGINE=InnoDB AUTO_INCREMENT=16 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='权限表';

CREATE TABLE `t_role` (
                          `uid` bigint NOT NULL AUTO_INCREMENT COMMENT '主键',
                          `value` varchar(32) NOT NULL COMMENT '角色值',
                          `name` varchar(64) NOT NULL COMMENT '角色名称',
                          `creator_id` bigint NOT NULL COMMENT '创建人id',
                          `create_time` timestamp NOT NULL DEFAULT (now()) COMMENT '创建时间',
                          `updater_id` bigint DEFAULT NULL COMMENT '更新人id',
                          `update_time` timestamp NULL DEFAULT NULL COMMENT '更新时间',
                          PRIMARY KEY (`uid`),
                          UNIQUE KEY `t_role_value_uindex` (`value`)
) ENGINE=InnoDB AUTO_INCREMENT=20 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='角色表';

CREATE TABLE `t_role_menu` (
                               `role_id` bigint NOT NULL COMMENT '用户uid',
                               `menu_id` bigint NOT NULL COMMENT '菜单uid',
                               UNIQUE KEY `t_user_menu_role_id_menu_id_uindex` (`role_id`,`menu_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='角色权限关联表';

CREATE TABLE `t_user` (
                          `uid` bigint NOT NULL AUTO_INCREMENT COMMENT '用户id',
                          `account` varchar(64) NOT NULL COMMENT '账号',
                          `name` varchar(64) NOT NULL COMMENT '用户名',
                          `password` varchar(128) NOT NULL COMMENT '密码',
                          `creator_id` bigint NOT NULL COMMENT '创建人id',
                          `create_time` timestamp NOT NULL DEFAULT (now()) COMMENT '创建时间',
                          `updater_id` bigint DEFAULT NULL COMMENT '更新人id',
                          `update_time` timestamp NULL DEFAULT NULL COMMENT '更新时间',
                          `avatar` varchar(32) DEFAULT NULL COMMENT '头像fileId',
                          PRIMARY KEY (`uid`),
                          UNIQUE KEY `t_user_account_uindex` (`account`)
) ENGINE=InnoDB AUTO_INCREMENT=100 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户表';

CREATE TABLE `t_user_role` (
                               `user_id` bigint NOT NULL COMMENT '用户uid',
                               `role_id` bigint NOT NULL COMMENT '角色uid',
                               UNIQUE KEY `t_user_role_role_id_user_id_uindex` (`role_id`,`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户角色关联表';

