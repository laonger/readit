
use arrow_array::{
    types::Float32Type,
    PrimitiveArray,
};

use crate::env;

pub fn pooling(dim: usize, input: &PrimitiveArray<Float32Type>) -> PrimitiveArray<Float32Type> {

    let stride = input.len()/dim;

    let input_width = input.len(); // 输入特征图的宽度
    //
    let  pool_width = stride;

    let output_width = (input_width - pool_width) / stride + 1;

    let mut output = vec![0.0; output_width];

    // 执行平均池化操作
    for i in 0..output_width {
        // 计算当前池化窗口的起始坐标
        let start_i = i * stride;

        // 提取池化窗口内的元素并计算平均值
        let mut sum = 0.0;
        for m in 0..pool_width {
            sum += input.value(start_i + m);
        }

        // 计算池化窗口的平均值并赋值给输出特征图
        output[i] = sum / pool_width as f32;
    }
    
    PrimitiveArray::<Float32Type>::from_iter_values(output.iter().copied())
}

// 测试平均池化层
//fn test() {
//    // 创建一个平均池化层，池化窗口大小为 (2, 2)，步长为 2
//    let avg_pooling_layer = AveragePoolingLayer::new((2, 2), 2);
//
//    // 输入特征图（4x4）
//    let input = vec![
//        vec![1.0, 3.0, 2.0, 4.0],
//        vec![5.0, 6.0, 8.0, 7.0],
//        vec![3.0, 2.0, 4.0, 1.0],
//        vec![8.0, 7.0, 3.0, 5.0],
//    ];
//
//    // 进行平均池化操作
//    let output = avg_pooling_layer.forward(&input);
//
//    // 打印输出
//    println!("Average Pooling Output: {:?}", output);
//}
