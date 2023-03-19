/*
 * @Author: ChiYu 1875406398@qq.com
 * @Date: 2022-11-09 11:33:26
 * @LastEditors: ChiYu 1875406398@qq.com
 * @LastEditTime: 2022-11-10 10:54:18
 * @FilePath: /async_await/src/main.rs
 * @Description: 验证异步
 */
use async_std::task::{sleep, spawn};
use std::time::Duration;

use futures::executor::block_on;

struct Song {
    author: String,
    name: String,
}

async fn learn_song() -> Song {
    Song {
        author: "曲婉婷".to_string(),
        name: String::from("《我的歌声里》"),
    }
}

async fn sing_song(song: Song) {
    println!(
        "给大家献上一首{}的{} ~ {}",
        song.author, song.name, "你存在我深深的脑海里~ ~"
    );
}

async fn dance() {
    println!("唱到情深处，身体不由自主的动了起来~ ~");
}

async fn learn_and_sing() {
    // 这里使用`.await`来等待学歌的完成，但是并不会阻塞当前线程，该线程在学歌的任务`.await`后，完全可以去执行跳舞的任务
    let song = learn_song().await;
    sleep(Duration::from_secs(5)).await;
    // 唱歌必须要在学歌之后
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!`可以并发的处理和等待多个`Future`，若`learn_and_sing Future`被阻塞，那`dance Future`可以拿过线程的所有权继续执行。若`dance`也变成阻塞状态，那`learn_and_sing`又可以再次拿回线程所有权，继续执行。
    // 若两个都被阻塞，那么`async main`会变成阻塞状态，然后让出线程所有权，并将其交给`main`函数中的`block_on`执行器
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}


trait SimpleFuture {
  type Output;
  fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
  Ready(T),
  Pending,
}


pub struct SocketRead<'a> {
  socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
  type Output = Vec<u8>;

  fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
      if self.socket.has_data_to_read() {
          // socket有数据，写入buffer中并返回
          Poll::Ready(self.socket.read_buf())
      } else {
          // socket中还没数据
          //
          // 注册一个`wake`函数，当数据可用时，该函数会被调用，
          // 然后当前Future的执行器会再次调用`poll`方法，此时就可以读取到数据
          self.socket.set_readable_callback(wake);
          Poll::Pending
      }
  }
}


/// 一个SimpleFuture，它会并发地运行两个Future直到它们完成
///
/// 之所以可以并发，是因为两个Future的轮询可以交替进行，一个阻塞，另一个就可以立刻执行，反之亦然
pub struct Join<FutureA, FutureB> {
  // 结构体的每个字段都包含一个Future，可以运行直到完成.
  // 如果Future完成后，字段会被设置为 `None`. 这样Future完成后，就不会再被轮询
  a: Option<FutureA>,
  b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
  FutureA: SimpleFuture<Output = ()>,
  FutureB: SimpleFuture<Output = ()>,
{
  type Output = ();
  fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
      // 尝试去完成一个 Future `a`
      if let Some(a) = &mut self.a {
          if let Poll::Ready(()) = a.poll(wake) {
              self.a.take();
          }
      }

      // 尝试去完成一个 Future `b`
      if let Some(b) = &mut self.b {
          if let Poll::Ready(()) = b.poll(wake) {
              self.b.take();
          }
      }

      if self.a.is_none() && self.b.is_none() {
          // 两个 Future都已完成 - 我们可以成功地返回了
          Poll::Ready(())
      } else {
          // 至少还有一个 Future 没有完成任务，因此返回 `Poll::Pending`.
          // 当该 Future 再次准备好时，通过调用`wake()`函数来继续执行
          Poll::Pending
      }
  }
}