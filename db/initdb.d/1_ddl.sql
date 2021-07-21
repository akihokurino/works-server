CREATE SCHEMA IF NOT EXISTS `works` DEFAULT CHARACTER SET utf8mb4;
USE `works`;

CREATE TABLE IF NOT EXISTS `users` (
    `id` VARCHAR(255) NOT NULL,
    `misoca_refresh_token` VARCHAR(255) NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    PRIMARY KEY (`id`)
)
ENGINE = InnoDB DEFAULT CHARSET=utf8mb4
COMMENT = '';

CREATE TABLE IF NOT EXISTS `suppliers` (
    `id` VARCHAR(255) NOT NULL,
    `user_id` VARCHAR(255) NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `billing_amount` INT(11) NOT NULL,
    `billing_type` INT(11) NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    PRIMARY KEY (`id`),
    INDEX `fk_suppliers_users_idx` (`user_id` ASC),
    CONSTRAINT `fk_suppliers_users`
        FOREIGN KEY (`user_id`)
        REFERENCES `users` (`id`)
        ON DELETE NO ACTION
        ON UPDATE NO ACTION
)
ENGINE = InnoDB DEFAULT CHARSET=utf8mb4
COMMENT = '';

CREATE TABLE IF NOT EXISTS `invoices` (
    `id` VARCHAR(255) NOT NULL,
    `supplier_id` VARCHAR(255) NOT NULL,
    `issue_ymd` VARCHAR(255) NOT NULL,
    `issue_at` DATE NULL,
    `payment_due_on_ymd` VARCHAR(255) NOT NULL,
    `payment_due_on_at` DATE NULL,
    `invoice_number` VARCHAR (255) NOT NULL,
    `payment_status` INT(11) NOT NULL,
    `invoice_status` INT(11) NOT NULL,
    `recipient_name` VARCHAR(255) NOT NULL,
    `subject` VARCHAR(255) NOT NULL,
    `total_amount` INT(11) NOT NULL,
    `tax` INT(11) NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    PRIMARY KEY (`id`),
    INDEX `fk_invoices_suppliers_idx` (`supplier_id` ASC),
    CONSTRAINT `fk_invoices_suppliers`
        FOREIGN KEY (`supplier_id`)
        REFERENCES `suppliers` (`id`)
        ON DELETE NO ACTION
        ON UPDATE NO ACTION
)
ENGINE = InnoDB DEFAULT CHARSET=utf8mb4
COMMENT = '';