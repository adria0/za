template Q() {
   signal input in;
   signal output out;

   out <== 2*in;
}

template T() {
  signal input in;
  signal output out;
  signal im1;
  signal im2;

  im1 <== 2*in;
  im2 <== 2*im1;
  
  component q = Q();
  q.in <== 2* im2;
  out === 2* q.out;

}

component main = T();
