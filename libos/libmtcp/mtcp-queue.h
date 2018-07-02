// -*- mode: c++; c-file-style: "k&r"; c-basic-offset: 4 -*-
/***********************************************************************
 *
 * include/posix-queue.h
 *   Zeus posix-queue interface
 *
 * Copyright 2018 Irene Zhang  <irene.zhang@microsoft.com>
 *
 * Permissposixn is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentatposixn
 * files (the "Software"), to deal in the Software without
 * restrictposixn, including without limitatposixn the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditposixns:
 *
 * The above copyright notice and this permissposixn notice shall be
 * included in all copies or substantial portposixns of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTPOSIXN OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTPOSIXN WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 **********************************************************************/
 
#ifndef _LIB_MTCP_QUEUE_H_
#define _LIB_MTCP_QUEUE_H_

#include "common/queue.h"
#include "common/library.h"
#include <list>
#include <map>

#define MTCP_MAX_FLOW_NUM  (10000)
#define MTCP_MAX_EVENTS (MTCP_MAX_FLOW_NUM * 3)
#define MTCP_RCVBUF_SIZE (2*1024)
#define MTCP_SNDBUF_SIZE (8*1024)


namespace Zeus {
namespace MTCP {

class MTCPQueue : public Queue {
private:
    struct PendingRequest {
    public:
        bool isDone;
        ssize_t res;
        // header = MAGIC, dataSize, SGA_num
        uint64_t header[3];
        // currently used incoming buffer
        void *buf;
        // number of bytes processed so far
        size_t num_bytes;
        struct sgarray sga;

        PendingRequest() :
            isDone(false),
            res(0),
            header{0,0,0},
            buf(NULL),
            num_bytes(0) { };            
    };
    
    // queued scatter gather arrays
    std::map<qtoken, PendingRequest> pending;
    std::list<qtoken> workQ;
    // queue and events
    uint32_t mtcp_evts;

    void ProcessIncoming(PendingRequest &req);
    void ProcessOutgoing(PendingRequest &req);
    void ProcessQ(size_t maxRequests);
    ssize_t Enqueue(qtoken qt, sgarray &sga);

public:
    MTCPQueue() : Queue(), workQ{} {
        mtcp_evts = 0;
    };
    MTCPQueue(BasicQueueType type, int qd) :
        Queue(type, qd), workQ{}  {
            mtcp_evts = 0;
        };

    // network functions
    static int queue(int domain, int type, int protocol);
    int listen(int backlog);
    int bind(struct sockaddr *saddr, socklen_t size);
    int accept(struct sockaddr *saddr, socklen_t *size);
    int connect(struct sockaddr *saddr, socklen_t size);
    int close();
          
    // file functions
    static int open(const char *pathname, int flags);
    static int open(const char *pathname, int flags, mode_t mode);
    static int creat(const char *pathname, mode_t mode);

    // data path functions
    ssize_t push(qtoken qt, struct sgarray &sga); // if return 0, then already complete
    ssize_t pop(qtoken qt, struct sgarray &sga); // if return 0, then already complete
    ssize_t wait(qtoken qt, struct sgarray &sga);
    ssize_t poll(qtoken qt, struct sgarray &sga);
    // returns the file descriptor associated with
    // the queue descriptor if the queue is an io queue
    int fd();
};

} // namespace MTCP 
} // namespace Zeus
#endif /* _LIB_MTCP_QUEUE_H_ */
