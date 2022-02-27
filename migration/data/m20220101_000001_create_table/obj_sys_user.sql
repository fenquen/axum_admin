/*
SQLyog Ultimate
MySQL - 10.6.5-MariaDB-1:10.6.5+maria~focal : Database - wk3
*********************************************************************
*/

/*!40101 SET NAMES utf8 */;

/*!40101 SET SQL_MODE=''*/;

/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
/*Data for the table `sys_user` */

insert  into `sys_user`(`id`,`user_name`,`user_nickname`,`user_password`,`user_salt`,`user_status`,`user_email`,`sex`,`avatar`,`role_id`,`dept_id`,`remark`,`is_admin`,`phone_num`,`last_login_ip`,`last_login_time`,`created_at`,`updated_at`,`deleted_at`) values 
('00TV876BOIIDCR9H7JA1KNNIGH','lingdu','神马','e89f83a1bbf67ee9a0bacdae21847166','j$~&)rL&0%','1','waong2005@126.com','1','','00UHIKGRA7JVIEU25NNGI8KTJU','00UHIKGR9LVRU8A25NNILKEVH1','及建原已参222','1','18613742567','',NULL,'2022-01-15 09:50:49','2022-02-18 22:18:45',NULL),
('00TV87DDOBJPU75J4TGUOC3NNG','admin','马磊','58daceaa16c143681b798ae9b8b2151f','FAq*yU3rqE','1','g.hssckwod@nku.bm','2','','00UHIKGRA7JVIF025NNH39CPMT','00UHIKGR9LVRU8A25NNILKEVH1','及建原已参','1','18613742567','',NULL,'2022-01-15 09:51:04','2022-02-20 15:32:28',NULL),
('00UGHLA3A1DR0GC7TLKCA32KK6','erwerw','fsvfs','4de1151c10a740d8c140c882cd7a2aa9','devVXhv9R~','1','waong2005@126.com','1','',NULL,'00UHIKGR9LVRU8A25NNILKEVH1','cvdsv','1','13334243252','',NULL,'2022-01-28 20:14:59','2022-02-18 22:38:34',NULL),
('00UT9J78PSU5QJRE3HSDUG94R2','user','user','4dc7373ccd817d86302e4c08d7e48813','qGUKfTRqPp','1','1@1.com','2','','00UHKP89CT1NDVFN6Q0E8LO7NT','00UHIKGR9LVRU8A25NNILKEVH1','普通用户','1','13312345678','',NULL,'2022-02-07 17:53:20','2022-02-20 15:48:07',NULL);

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
