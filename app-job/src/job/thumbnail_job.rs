// use common::AppError;
// use image::ImageReader;
// use model::{FileInfoTem, FileType};
// use sqlx::MySqlPool;
// use std::fs::File;
// use std::io::{Cursor, Read, Write};
// use std::path::Path;
// use uuid::Uuid;
// use webp::Encoder;
// 
// const CHUNK_SIZE: usize = 1 * 100 * 1024;
// 
// async fn do_task(conn: &MySqlPool) -> Result<(), AppError> {
//     let file_list = query_thumbnail_file(conn, FileType::IMAGE).await?;
//     for file_item in file_list {
//         let mut buffer = Vec::with_capacity(file_item.size as usize);
//         let file_split: Vec<&str> = file_item.items.split(",").collect();
//         let mut file_dir = None;
//         for item in file_split {
//             let mut file = File::open(item)?;
//             file.read_to_end(&mut buffer)?;
//             if file_dir.is_none() {
//                 let parent_dir = Path::new(item).parent().unwrap();
//                 let x = parent_dir.to_str().unwrap();
//                 file_dir = Some(x);
//             }
//         }
//         let img = ImageReader::new(Cursor::new(buffer))
//             .with_guessed_format()
//             .unwrap() // 自动识别图片格式
//             .decode()
//             .unwrap();
//         let encoder = Encoder::from_image(&img).unwrap();
//         let webp_data = encoder.encode(75.0); // 75% 质量
//         let fid = Uuid::new_v4();
//         let thumbnail_path = format!("{}/{}", file_dir.unwrap(), fid.clone().to_string());
//         let mut file = File::create(&thumbnail_path).unwrap();
//         file.write_all(&webp_data)?;
//         change_thumbnail_file(conn,&file_item.id,&thumbnail_path).await?;
//     }
//     Ok(())
// }
// 
// async fn query_thumbnail_file(
//     conn: &MySqlPool,
//     file_type: FileType,
// ) -> Result<Vec<FileInfoTem>, AppError> {
//     let list_db = sqlx::query_as::<_, FileInfoTem>(
//         "SELECT id,file_name, file_type, image_type, items, thumbnail, size, thumbnail_status FROM file_info where thumbnail_status=? and file_type=?"
//     )
//         .bind(true)
//         .bind(file_type)
//         // ✅ 绑定参数
//         .fetch_all(conn)
//         .await.unwrap();
//     Ok(list_db)
// }
// 
// async fn change_thumbnail_file(
//     conn: &MySqlPool,
//     id: &String,
//     thumbnail_path: &String,
// ) -> Result<(), AppError> {
//     sqlx::query("update file_info set thumbnail_status=0,thumbnail=? where id=?")
//         .bind(thumbnail_path)
//         .bind(id)
//         .execute(conn)
//         .await
//         .unwrap();
//     Ok(())
// }
