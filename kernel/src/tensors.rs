
use core::ops::Add;

struct TensorType<T, const N: usize, const S: usize> 
{
    data: [T; N],
    shape: [usize; S]
}




impl<T, const N: usize, const S: usize>  TensorType<T, N, S>
{
    pub fn New(data: [T; N], shape: [usize; S]) -> Self
    {
        TensorType {
            data,
            shape
        }
    }

    pub fn Rank(&self) -> usize
    {
        S
    }


    //index conversor unidimencional for N-dimencional
    //this function not change the tensor
    pub fn CUFN(&self, mut i: usize) -> [usize; S] 
    {
        let mut indexs = [0; S];

        for d in (0..S).rev()
        {
            indexs[d] = i % self.shape[d];
            i /= self.shape[d];
        } 

        indexs
    }

    pub fn rev_CUFN(&self, indexs: [usize; S]) -> usize 
    {
        let mut indice_linear = 0;
        let mut stride = 1; 

        for d in (0..S).rev() 
        {
            indice_linear += indexs[d] * stride;
            stride *= self.shape[d];
        }

        indice_linear
    }

    pub fn map<F>(&mut self, func: F) -> Self
    where 
        F: Fn(usize, T) -> T,
        T: Copy,
    {
        for i in 0..N
        {
            self.data[i] = func(i, self.data[i]);
        }

        self
    }
}